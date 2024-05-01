import os, shutil, json

current_path = os.getcwd()

with open(f"{current_path}/package/Cargo.toml", "r") as file:
    lines = file.readlines()

    for line in lines:
        if line[0:4] == "name":
            break

    start = line.find('"')
    end = line.find('"', start +1)
    pkg_name = line[start+1: end]

contract_name = input("insert contract name: ")

#region workspace

with open(f"{current_path}/Cargo.toml", "r") as file:
    lines = file.readlines()

for (i, line) in enumerate(lines):
    if line[0:7] == "members":
        break

start = line.find("[")
end = line.find("]")
content = line[start: end+1]

content_l: list = json.loads(content)
content_l.append(f"contracts/{contract_name}")
content = json.dumps(content_l)
content = f"{line[:start]}{content}\n"
lines[i] = content
        
with open(f"{current_path}/Cargo.toml", "w") as file:
    file.writelines(lines)

#endregion

#region Copy files 

new_path = f"{current_path}/contracts/{contract_name}"

shutil.copytree(f"{current_path}/template/contract", new_path)

with open(f"{new_path}/Cargo.toml", "r") as file:
    lines = file.read()

lines = lines.replace("$contract$", contract_name)

with open(f"{new_path}/Cargo.toml", "w") as file:
    file.write(lines)

with open(f"{new_path}/Cargo.toml", "r") as file:
    lines = file.readlines()

lines.insert(len(lines)-3, f"{pkg_name}{' ' * max(0, 17 - len(pkg_name))}" + "= { workspace = true }\n" )

with open(f"{new_path}/Cargo.toml", "w") as file:
    file.writelines(lines)

with open(f"{new_path}/src/contract.rs", "r") as file:
    lines = file.read()

lines = lines.replace("$pkg$", pkg_name.replace("-", "_"))
lines = lines.replace("$contract$", contract_name)

with open(f"{new_path}/src/contract.rs", "w") as file:
    file.write(lines)

#endregion

#region package
new_path = f"{current_path}/package"
shutil.copyfile(f"{current_path}/template/package/package.rs", f"{new_path}/src/{contract_name}.rs")

with open(f"{new_path}/src/lib.rs", "r") as file:
    lines = file.readlines()

lines.append(f"pub mod {contract_name};\n")

with open(f"{new_path}/src/lib.rs", "w") as file:
    file.writelines(lines)

#endregion
