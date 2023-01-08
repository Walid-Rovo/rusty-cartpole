use mem_macros::size_of;
use bus::{Bus, BusReader};
use std::{
    sync::mpsc::{channel, Receiver, Sender},
    thread,
    time::{Duration, SystemTime},
};
use ticker::Ticker;

#[allow(dead_code)]
pub struct ThreadableCartPole {
    state: [f32; 4],
    state_bus: Bus<[f32; 4]>,
    action_receiver: Receiver<f32>,
    action_sender: Sender<f32>,
    mp: f32,
    mc: f32,
    lp: f32,
    dt: f32,
    dt_dur: Duration,
    spinner_kill: bool,
}

#[allow(dead_code)]
impl ThreadableCartPole {
    pub fn new(
        x0: f32,
        xdot0: f32,
        theta0: f32,
        thetadot0: f32,
        mp: f32,
        mc: f32,
        lp: f32,
        dt: Option<f32>,
    ) -> ThreadableCartPole {
        let (action_sender, action_receiver) = channel::<f32>();
        let dt = match dt {
            Some(val) => val,
            None => 0.012_f32,
        };
        return ThreadableCartPole {
            state: [x0, xdot0, theta0, thetadot0],
            state_bus: Bus::new(size_of!([f32; 4])),
            action_receiver,
            action_sender,
            mp,
            mc,
            lp,
            dt,
            dt_dur: Duration::from_secs_f32(dt),
            spinner_kill: false,
        };
    }
    fn step(&mut self) {
        let action = match self.action_receiver.recv_timeout(self.dt_dur) {
            Ok(action) => action,
            Err(_) => 0_f32,
        };
        const G: f32 = 9.81;
        // Pre-calc common params
        let costheta = self.state[2].cos();
        let sintheta = self.state[2].sin();
        let mplp = self.mp * self.lp;
        let mc_mp = self.mc + self.mp;
        // self.mp * self.lp * sintheta * self.state[3].powi(2)
        let den0 = mplp * sintheta * self.state[3].powi(2);

        let x_ddot = (action + den0 + self.mp * G * costheta * sintheta)
            / (mc_mp - self.mp * costheta.powi(2));
        let theta_ddot = (action * costheta - (mc_mp) * G * sintheta + costheta * den0)
            / (mplp * costheta.powi(2) - (mc_mp) * self.lp);
        // {   // full equations
        //     let [mut x, mut x_dot, mut theta, mut theta_dot]: [f32; 4] =
        //         self.state.clone().try_into().ok().unwrap();
        //     let x_ddot = (action
        //         + self.mp * self.lp * sintheta * theta_dot.powi(2)
        //         + self.mp * 9.81 * costheta * sintheta)
        //         / (self.mc + self.mp - self.mp * costheta.powi(2));
        //     let theta_ddot = (action * costheta - (self.mc + self.mp) * 9.81 * sintheta
        //         + self.mp * self.lp * costheta * sintheta * theta_dot.powi(2))
        //         / (self.mp * self.lp * costheta.powi(2) - (self.mc + self.mp) * self.lp);
        // }

        // Semi-implicit Euler integration
        self.state[1] = self.state[1] + self.dt * x_ddot;
        self.state[0] = self.state[0] + self.dt * self.state[1];
        self.state[3] = self.state[3] + self.dt * theta_ddot;
        self.state[2] = self.state[2] + self.dt * self.state[3];

        self.state_bus.broadcast(self.state);
    }
    pub fn get_action_sender(&mut self) -> Sender<f32> {
        self.action_sender.clone()
    }
    pub fn get_state_bus_receiver(&mut self) -> BusReader<[f32; 4]> {
        self.state_bus.add_rx()
    }
    pub fn stop_spinner(mut self) {
        self.spinner_kill = true;
    }
    pub fn start_spinner(mut self) {
        let now = SystemTime::now();
        let mut delta_t = now.elapsed().unwrap();

        let dur = self.dt_dur.clone();
        let mut i = 0;
        let _ = thread::spawn(move || {
            let mut infinite_ticker_iter = Ticker::new(0.., dur).into_iter();
            // let mut infinite_ticker_iter = 0..;
            while let Some(_) = infinite_ticker_iter.next() {
                if cfg!(debug_assertions) {
                    let prev_iter_dur = (now.elapsed().unwrap() - delta_t).as_secs_f32();
                    let dt_warn_limit = self.dt * 1.1;
                    if prev_iter_dur > dt_warn_limit {
                        println!(
                            "Iter {i} took more than {}ms, {}ms\nIt was supposed to take {}ms\n",
                            dt_warn_limit * 1000.0,
                            prev_iter_dur * 1000.0,
                            self.dt * 1000.0,
                        );
                    }
                    delta_t = now.elapsed().unwrap();
                    i += 1;
                }
                if self.spinner_kill {
                    drop(infinite_ticker_iter);
                    self.spinner_kill = false;
                    break;
                }
                self.step();
                ()
            }
        });
    }
    pub fn reset(mut self) {
        self.state = [0.; 4];
    }
}
