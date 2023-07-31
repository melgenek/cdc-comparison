#!/bin/sh

mkdir -p extracted

cat postgres-15.2-partaa postgres-15.2-partab | tar -xzv -C extracted
cat postgres-15.3-partaa postgres-15.3-partab | tar -xzv -C extracted
