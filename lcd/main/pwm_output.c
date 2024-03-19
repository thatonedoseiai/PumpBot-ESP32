#include "pwm_output.h"

pb_output_info_t* pwms;
gptimer_handle_t pwm_irq_timer;

void update_pwm(int channel) {
	if(atomic_load(&pwms[channel].off)) {
		ledc_stop(LEDC_LOW_SPEED_MODE, channel, 0);
		return;
	}
	pwms[channel].was_updated = 1;
	uint16_t power = output_get_value(channel);
	ledc_set_duty_and_update(LEDC_LOW_SPEED_MODE, channel, power, 0);
}

static bool pwm_callback(gptimer_handle_t timer, const gptimer_alarm_event_data_t* edata, void* user_ctx) {
	for(int i=0;i<4;++i) {
		uint16_t k = atomic_load(&(pwms[i].cyclesLeft));
		if(k == 1) {
			uint16_t b = atomic_load(&(pwms[i].old_output));
			atomic_store(&(pwms[i].output), b);
			update_pwm(i);
		}
		if(k) {
			atomic_fetch_sub(&(pwms[i].cyclesLeft), 1);
			// ets_printf("channel %d: %d\n", k-1);
		}
	}
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
		.alarm_count = 20000,
		.flags.auto_reload_on_alarm = true,
	};
	gptimer_event_callbacks_t cbs = { .on_alarm = pwm_callback };
	ESP_ERROR_CHECK(gptimer_new_timer(&timer_config, &pwm_irq_timer));
	ESP_ERROR_CHECK(gptimer_set_alarm_action(pwm_irq_timer, &alarm_config));
	ESP_ERROR_CHECK(gptimer_register_event_callbacks(pwm_irq_timer, &cbs, NULL));
	ESP_ERROR_CHECK(gptimer_enable(pwm_irq_timer));
	ESP_ERROR_CHECK(gptimer_start(pwm_irq_timer));
}

void done_pb_output_info() {
	gptimer_del_timer(pwm_irq_timer);
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
	pwms[channel].cyclesLeft = 0;
	update_pwm(channel);
}

// cs = 1 means 100 cs = 0.1s on
void output_set_value_timeout(int channel, int level, int cs) {
	if(atomic_load(&pwms[channel].off))
		return;
	uint16_t k = atomic_exchange(&pwms[channel].output, level);
	atomic_store(&pwms[channel].old_output, k);
	
	ledc_set_duty_and_update(LEDC_LOW_SPEED_MODE, channel, level, 0);
	pwms[channel].cyclesLeft = cs*5;
	pwms[channel].was_updated = 1;
}

int output_get_value(int channel) {
	return atomic_load(&(pwms[channel].output));
}

char is_off(int channel) {
	return atomic_load(&(pwms[channel].off));
}

void output_add_value(int channel, int increment) {
	if(increment > 0)
		atomic_fetch_add(&(pwms[channel].output), increment);
	else if(atomic_load(&(pwms[channel].output)) >= -increment)
		atomic_fetch_sub(&(pwms[channel].output), -increment);
	if(atomic_load(&(pwms[channel].output)) > 16383)
		atomic_store(&(pwms[channel].output), 16383);
	pwms[channel].cyclesLeft = 0;
	update_pwm(channel);
}

char output_was_updated(int channel) {
	char x = pwms[channel].was_updated;
	pwms[channel].was_updated = 0;
	return x;
}
