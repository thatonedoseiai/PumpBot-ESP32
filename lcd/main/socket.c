#include "socket.h"
#include "system_status.h"
#include <rom/ets_sys.h>

#include <sys/socket.h>
#include <unistd.h>
#include <arpa/inet.h>
#include <pthread.h>
#include <errno.h>

int sockfd;
pthread_t ping_thread_id;
char ping_flag;				// flag to control the thread because joining doesn't work
pthread_mutex_t sockfd_lock;
char run_manual_heartbeat;	// flag to ONLY run the manual heartbeat IF there is not enough activity on the network

void* ping(void* arg) {
	ets_printf("started ping server!\n");
	while(ping_flag) {
		ets_printf("ping!\n");
		int e = send_message("p\n", 2);
		ets_printf("sent characters: %d\n", e);
		char buffer[8];
		vTaskDelay(pdMS_TO_TICKS(10000));
		if(run_manual_heartbeat) {
			e = get_message(buffer, 8);
			if(e < 0) {
				ets_printf("error_val: %s (%d)\n", strerror(errno), errno);
			// if(buffer[0] == 0) {
				ets_printf("died! response: %d\n", buffer[0]);
				ping_flag = false;
				disconnect_from_server();
				system_flags &= ~FLAG_SERVER_CONNECTED;
				// pthread_mutex_destroy(&sockfd_lock);
				// break;
				pthread_exit(NULL);
			}
			ets_printf("pong!\n");
			run_manual_heartbeat = true;
			buffer[0] = 0;
		}
		do {
			run_manual_heartbeat = true;
			vTaskDelay(pdMS_TO_TICKS(5000));
			ets_printf("waiting!\n");
		} while(!run_manual_heartbeat);
	}
	ets_printf("leaving!\n");
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
	ping_flag = true;
	run_manual_heartbeat = true;

	pthread_mutex_init(&sockfd_lock, NULL);
	pthread_create(&ping_thread_id, NULL, &ping, NULL);
	return 0;
}

int get_message(char* buffer, int buflen) {
	if(!ping_flag)
		return -1;		// not connected to server
	pthread_mutex_lock(&sockfd_lock);
	int k = recv(sockfd, buffer, buflen, MSG_DONTWAIT);
	pthread_mutex_unlock(&sockfd_lock);
	if(k > 0)
		run_manual_heartbeat = false;
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
		ets_printf("killing the ping thread!\n");
		ping_flag = false;
		// if(ping_thread_id != pthread_self())
		// 	(void) pthread_join(ping_thread_id, NULL);
	}
	shutdown(sockfd, SHUT_RDWR);
	pthread_mutex_unlock(&sockfd_lock);
	pthread_mutex_destroy(&sockfd_lock);
	return close(sockfd);
}
