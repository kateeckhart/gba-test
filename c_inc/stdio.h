#ifndef STDIO_H
#define STDIO_H

#include <stdarg.h>
#include <stdlib.h>

int sprintf(char* str, const char* format, ...);
int snprintf(char* str, size_t n, const char* format, ...);
int vsprintf(char* str, const char* format, va_list va);
int vsnprintf(char* str, size_t n, const char* format, va_list va);
#endif
