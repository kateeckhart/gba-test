#ifndef FCNTL_H
#define FCNTL_H

typedef int off_t;
typedef int mode_t;

#define O_RDONLY 1
#define O_WRONLY 2
#define O_RDRW 3
#define O_CREAT 4
#define O_TRUNC 8
#define O_APPEND 16

#define SEEK_SET 0
#define SEEK_CUR 1
#define SEEK_END 2

int close(int fd);
int open(const char *pathname, int flags, mode_t mode);
ssize_t read(int fd, void *buf, size_t count);
ssize_t write(int fd, const void *buf, size_t count);

off_t lseek(int fd, off_t offset, int whence);

#endif
