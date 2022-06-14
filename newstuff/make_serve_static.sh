clang -Weverything generate_response_file.c -o generate_response_file
./generate_response_file $1
rm generate_response_file
clang -Weverything serve_static.c -o serve_static
rm response.h