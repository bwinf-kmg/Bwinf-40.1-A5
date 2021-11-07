#!/usr/bin/env bash

for i in {0..5} ; do
    ./Aufgabe5-Rust "beispiele/gewichtsstuecke${i}.txt" "ergebnisse/loesung${i}.txt"
done