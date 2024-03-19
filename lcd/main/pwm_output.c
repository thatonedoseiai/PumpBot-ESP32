#include "pwm_output.h"

pb_output_info_t* pwms;

static bool pwm_callback(gptimer_handle_t timer, const gptimer_alarm_event_data_t* edata, void* user_ctx) {
	pb_output_info_t* p = (pb_output_info_t*) user_ctx;
	if(p->has_timer)
		atomic_store(&p->output, p->prev_value);
	p->has_timer = 0;
	return true;
}

void init_pb_output_info() {
	pwms = calloc(4, sizeof(pb_output_info_t));
	gptimer_config_t timer_config = {
		.clk_src = GPTIMER_CLK_SRC_DEFAULT, 
		.direction = GPTIMER_COUNT_UP,
		.resolution_hz = 1*1000*1000 // 1MHz
	};
	gptimer_alarm_config_t alarm_config = {
		.reload_count = 0,
		.alarm_count = 1000000,
		.flags.auto_reload_on_alarm = false,
	};
	gptimer_event_callbacks_t cbs = { .on_alarm = pwm_callback };
	for(int i=0;i<4;++i) {
		ESP_ERROR_CHECK(gptimer_new_timer(&timer_config, &pwms[i].timeout));
		ESP_ERROR_CHECK(gptimer_set_alarm_action(pwms[i].timeout, &alarm_config));
		ESP_ERROR_CHECK(gptimer_register_event_callbacks(pwms[i].timeout, &cbs, (void*) &pwms[i]));
	}
}

void update_pwm(int channel) {
	if(atomic_load(&pwms[channel].off)) {
		ledc_stop(LEDC_LOW_SPEED_MODE, channel, 0);
		return;
	}
	uint16_t power = output_get_value(channel);
	ledc_set_duty_and_update(LEDC_LOW_SPEED_MODE, channel, power, 0);
}

void done_pb_output_info() {
	for(int i=0;i<4;++i)
		gptimer_del_timer(pwms[i].timeout);
	free(pwms);
}

void output_set_power(int channel, char enable) {
	atomic_store(&(pwms[channel].off), enable);
	update_pwm(channel);
}

void output_toggle(int channel) {
	atomic_fetch_xor(&(pwms[channel].off), 1);
	update_pwm(channel);
}

void output_set_value(int channel, int level) {
	atomic_store(&(pwms[channel].output), level);
	pwms[channel].has_timer = 0;
	update_pwm(channel);
}

void output_set_value_timeout(int channel, int level, int ms) {
	gptimer_alarm_config_t alarm_config = {
		.reload_count = 0,
		.alarm_count = ms*1000,
		.flags.auto_reload_on_alarm = false,
	};
	ESP_ERROR_CHECK(gptimer_set_alarm_action(pwms[channel].timeout, &alarm_config));
	pwms[channel].has_timer = 1;
	gptimer_start(pwms[channel].timeout);
}

inline int output_get_value(int channel) {
	return atomic_load(&(pwms[channel].output));
}

inline char is_off(int channel) {
	return atomic_load(&(pwms[channel].off));
}

void output_add_value(int channel, int increment) {
	if(increment > 0)
		atomic_fetch_add(&(pwms[channel].output), increment);
	else
		atomic_fetch_sub(&(pwms[channel].output), increment);
	pwms[channel].has_timer = 0;
	update_pwm(channel);
}
