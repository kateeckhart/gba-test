#ifndef STDLIB_H
#define STDLIB_H
typedef unsigned int size_t;
typedef int ssize_t;

void* malloc(size_t count);
void free(void* ptr);
void* calloc(size_t count, size_t size);

#endif
