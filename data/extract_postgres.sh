#!/bin/sh

mkdir -p extracted
mkdir -p concatenated

cat postgres-15.2-partaa postgres-15.2-partab | tar -xzv -C extracted
cat postgres-15.3-partaa postgres-15.3-partab | tar -xzv -C extracted

LC_ALL=C gtar --sort=name --mtime=0 --owner=me:1111 --group=me:1111 -cvf concatenated/postgres-15.2.tar extracted/postgres-15.2-extracted
LC_ALL=C gtar --sort=name --mtime=0 --owner=me:1111 --group=me:1111 -cvf concatenated/postgres-15.3.tar extracted/postgres-15.3-extracted
