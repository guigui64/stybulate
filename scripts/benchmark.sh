#!/bin/bash
time for i in $(seq 10); do ./target/release/stybulate -1 -f fancy -o data/output.txt data/input.txt ; done
