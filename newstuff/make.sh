clang -Weverything gen.c -o gen
./gen $1
rm gen
clang -Weverything server.c -o server
