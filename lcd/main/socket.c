#include "socket.h"

#include <sys/socket.h>
#include <unistd.h>
#include <arpa/inet.h>

int sockfd;

int connect_to_server(uint32_t ip, uint16_t pt) {
	struct sockaddr_in serv_addr;
	sockfd = socket(AF_INET, SOCK_STREAM, 0);
	serv_addr.sin_family = AF_INET;
	serv_addr.sin_port = htons(pt);
	serv_addr.sin_addr.s_addr = ip;
	int k = connect(sockfd, (struct sockaddr*) &serv_addr, sizeof(serv_addr));
	if(k < 0)
		return k;
	return 0;
}

int get_message(char* buffer, int buflen) {
	return recv(sockfd, buffer, buflen, MSG_DONTWAIT);
}

int send_message(const char* buffer, int buflen) {
	return send(sockfd, buffer, buflen, MSG_DONTWAIT);
}

int disconnect_from_server(void) {
	return shutdown(sockfd, SHUT_RDWR);
}
