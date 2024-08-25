use std::{
    mem,
    sync::{Arc, RwLock, RwLockWriteGuard},
    thread,
    time::Instant,
};

use super::state::State;

#[derive(Default)]
pub struct ApplicationRuntime {}

impl ApplicationRuntime {
    pub fn start(&mut self, state_arc: &Arc<RwLock<State>>) {
        {
            let mut state = state_arc.write().unwrap();

            let mut subscribers = mem::take(&mut state.event_subscribers);

            println!("Starting event subscribers...");

            for subscriber in &mut subscribers {
                subscriber.start(&mut state);
            }

            state.event_subscribers.extend(subscribers);

            if state.stop {
                return;
            }
        }

        self.start_update_thread(Arc::clone(&state_arc));
        //self.start_fixed_update_thread(Arc::clone(&state_arc));
        self.start_main_thread(Arc::clone(&state_arc));
    }

    fn start_update_thread(&mut self, state_arc: Arc<RwLock<State>>) {
        //let state = Arc::clone(&state);

        thread::spawn(move || {
            let mut delta_time = 0u128;
            let mut delta_t = 0u128;

            let instant = Instant::now();

            let mut last = instant.elapsed().as_nanos();
            let mut last_tick = instant.elapsed().as_nanos();

            let mut delta_tick_time = 0f64;
            let mut measured_ticks_per_second = 0f64;

            loop {
                let mut state = state_arc.write().unwrap();

                state.delta_tick_time = delta_tick_time;
                state.measured_ticks_per_second = measured_ticks_per_second;

                if state.stop {
                    break;
                }

                if delta_t >= 1_000_000_000
                /*1e9u128*/
                {
                    delta_tick_time = (instant.elapsed().as_nanos() - last_tick) as f64 / 1.0e9f64;
                    last_tick = instant.elapsed().as_nanos();

                    measured_ticks_per_second = 1f64 / delta_tick_time;

                    ApplicationRuntime::update(&mut state);

                    delta_t -= 1_000_000_000;
                }

                delta_time = instant.elapsed().as_nanos() - last;
                last += delta_time;
                delta_t += delta_time * state.tps as u128;
            }
        });
    }

    fn start_fixed_update_thread(&mut self, state_arc: Arc<RwLock<State>>) {
        todo!();
    }

    fn start_main_thread(&mut self, state_arc: Arc<RwLock<State>>) {
        let mut delta_time = 0u128;
        let mut delta_t = 0u128;

        let instant = Instant::now();

        let mut last = instant.elapsed().as_nanos();
        let mut last_frame = instant.elapsed().as_nanos();

        let mut delta_frame_time = 0f64;
        let mut measured_frames_per_second = 0f64;

        loop {
            {
                let mut state = state_arc.write().unwrap();

                state.delta_frame_time = delta_frame_time;
                state.measured_frames_per_second = measured_frames_per_second;

                if state.stop {
                    break;
                }
            }

            if delta_t >= 1_000_000_000
            /*1e9u128*/
            {
                delta_frame_time = (instant.elapsed().as_nanos() - last_frame) as f64 / 1.0e9f64;
                last_frame = instant.elapsed().as_nanos();

                measured_frames_per_second = 1f64 / delta_frame_time;

                self.render(&mut state_arc.write().unwrap());

                delta_t -= 1_000_000_000;
            }

            delta_time = instant.elapsed().as_nanos() - last;
            last += delta_time;
            delta_t += delta_time * state_arc.read().unwrap().fps as u128;
        }
    }

    fn update(state: &mut RwLockWriteGuard<State>) {
        let mut subscribers = mem::take(&mut state.event_subscribers);

        for subscriber in &mut subscribers {
            subscriber.update(state);
        }

        state.event_subscribers.extend(subscribers);
    }

    fn fixed_update(state: &mut RwLockWriteGuard<State>) {
        let mut subscribers = mem::take(&mut state.event_subscribers);

        for subscriber in &mut subscribers {
            subscriber.fixed_update(state);
        }

        state.event_subscribers.extend(subscribers);
    }

    fn render(&mut self, state: &mut RwLockWriteGuard<State>) {
        let mut subscribers = mem::take(&mut state.event_subscribers);

        for subscriber in &mut subscribers {
            subscriber.render(state);
        }

        state.event_subscribers.extend(subscribers);
    }
}
