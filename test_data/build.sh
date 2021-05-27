#!/bin/bash

mkdir -p png
mkdir -p diff

for file in svg/*.svg; do
    name=${file#"svg/"}
    name=${name%".svg"}
    echo ${name}
    inkscape --export-png=png/${name}.png -b white -d 75 -z $file
done
