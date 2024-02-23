#ifndef UTF_8_H
#define UTF_8_H

#include <stddef.h>

int decode_code_point(char **s);
void encode_code_point(char **s, char *end, int code);
int split_into_surrogates(int code, int *surr1, int *surr2);
int split_into_surrogates(int code, int *surr1, int *surr2);
size_t count_utf8_code_points(const char *s);

#endif
