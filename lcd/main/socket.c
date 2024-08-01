#include "socket.h"
#include "system_status.h"
#include <rom/ets_sys.h>

#include <sys/socket.h>
#include <unistd.h>
#include <arpa/inet.h>
#include <pthread.h>

int sockfd;
pthread_t ping_thread_id;
char ping_flag;
pthread_mutex_t sockfd_lock;

void* ping(void* arg) {
	ets_printf("started ping server!\n");
	while(ping_flag) {
		send_message("p\n", 2);
		char buffer[8];
		ets_printf("ping!\n");
		vTaskDelay(pdMS_TO_TICKS(500));
		get_message(buffer, 8);
		if(buffer[0] != 200) {
			ets_printf("died!\n");
			disconnect_from_server();
			system_flags &= ~FLAG_SERVER_CONNECTED;
			ping_flag = false;
			pthread_mutex_destroy(&sockfd_lock);
			pthread_exit(NULL);
		}
		ets_printf("pong!\n");
		buffer[0] = 0;
		vTaskDelay(pdMS_TO_TICKS(5000));
	}
	return NULL;
}

int connect_to_server(uint32_t ip, uint16_t pt) {
	struct sockaddr_in serv_addr;
	sockfd = socket(AF_INET, SOCK_STREAM, 0);
	serv_addr.sin_family = AF_INET;
	serv_addr.sin_port = htons(pt);
	serv_addr.sin_addr.s_addr = ip;
	int k = connect(sockfd, (struct sockaddr*) &serv_addr, sizeof(serv_addr));
	if(k < 0)
		return k;
	pthread_mutex_init(&sockfd_lock, NULL);
	ping_flag = true;
	pthread_create(&ping_thread_id, NULL, &ping, NULL);
	return 0;
}

int get_message(char* buffer, int buflen) {
	pthread_mutex_lock(&sockfd_lock);
	int k = recv(sockfd, buffer, buflen, MSG_DONTWAIT);
	pthread_mutex_unlock(&sockfd_lock);
	return k;
}

int send_message(const char* buffer, int buflen) {
	pthread_mutex_lock(&sockfd_lock);
	int k = send(sockfd, buffer, buflen, MSG_DONTWAIT);
	pthread_mutex_unlock(&sockfd_lock);
	return k;
}

int disconnect_from_server(void) {
	pthread_mutex_lock(&sockfd_lock);
	if(ping_flag) {
		ets_printf("joining the ping thread!\n");
		ping_flag = false;
		pthread_mutex_unlock(&sockfd_lock);
		(void) pthread_join(ping_thread_id, NULL);
	}
	shutdown(sockfd, SHUT_RDWR);
	pthread_mutex_destroy(&sockfd_lock);
	return close(sockfd);
}
