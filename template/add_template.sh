#!/bin/bash

current_path=$(pwd)

pkg_name=""
while IFS= read -r line; do
  if [ "${line:0:4}" = "name" ]; then
    start=$(expr index "$line" '"')
    end=$(expr index "$line" '"' $((start+1)))
    pkg_name=${line:start:end-1}
    break
  fi
done < "$current_path/package/Cargo.toml"

read -p "insert contract name: " contract_name

# workspace

i=0
while IFS= read -r line; do
  if [ "${line:0:7}" = "members" ]; then
    break
  fi
  ((i++))
done < "$current_path/Cargo.toml"

start=$(expr index "$line" '[')
end=$(expr index "$line" ']')
content=${line:start:end-2}
content_arr=($content)
content_arr+=("contracts/$contract_name")
content=$(printf "%s," "${content_arr[@]}")
content="[ $content ]"
sed -i "$i s/.*/$content/" "$current_path/Cargo.toml"

# Copy files

new_path="$current_path/contracts/$contract_name"
cp -r "$current_path/template/contract" "$new_path"

sed -i "s/\$contract\$/$contract_name/g" "$new_path/Cargo.toml"

i=$(($(wc -l < "$new_path/Cargo.toml") - 3))
pkg_name_len=${#pkg_name}
spaces=$((17 - pkg_name_len))
if [ $spaces -lt 0 ]; then
  spaces=0
fi
pkg_line="$pkg_name$(printf ' %.0s' $(seq 1 $spaces)) = { workspace = true }"
sed -i "$i a $pkg_line" "$new_path/Cargo.toml"

sed -i "s/\$pkg\$/$(echo $pkg_name | sed 's/-/_/g')/g; s/\$contract\$/$contract_name/g" "$new_path/src/contract.rs"

# package

new_path="$current_path/package"
cp "$current_path/template/package/package.rs" "$new_path/src/$contract_name.rs"

while IFS= read -r line; do
  lines+=("$line")
done < "$new_path/src/lib.rs"

lines+=("pub mod $contract_name;")
printf "%s\n" "${lines[@]}" > "$new_path/src/lib.rs"
