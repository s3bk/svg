#!/bin/bash

for file in svg/*.svg; do
    name=${file#"svg/"}
    name=${name%".svg"}
    echo ${name}
    inkscape --export-png=png/${name}.png -d 75 -z $file
done
