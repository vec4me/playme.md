clang -Weverything static.c -o static
./static $1
rm static
clang -Weverything server.c -o server
