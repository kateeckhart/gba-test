#ifndef STRING_H
#define STRING_H

#include <stdlib.h>

void* memcpy(void* dest, const void* src, size_t count);
void* memmove(void* dest, const void* src, size_t count);
void* memset(void* dest, int byte, size_t count);
void* memchr(const void* ptr, int val, size_t size);

char* strcpy(char* dest, const char* src);
size_t strlen(const char *str);
char* strcat(char* destination, const char* source);
#endif
