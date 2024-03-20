#ifndef SOCKET_H
#define SOCKET_H

#include <stdint.h>

int connect_to_server(uint32_t ip, uint16_t pt);
int get_message(char* buffer, int buflen);
int send_message(char* buffer, int buflen);
int disconnect_from_server(void);

#endif
