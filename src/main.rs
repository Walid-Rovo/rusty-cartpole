mod cartpole_viewer;
mod cartpole_sim;

use crate::cartpole_viewer::CartPoleViewer;
use crate::cartpole_sim::ThreadableCartPole;

fn main() {
    const DT: f32 = 0.012; // at 8ms hits speed limit caused by the renderer
    let mut cartpole = ThreadableCartPole::new(0.0, 0.0, 0.0, 0.01, 0.23, 2.4, 0.36, Some(DT));
    let state_recv = cartpole.get_state_bus_receiver();
    cartpole.start_spinner();

    let mut app = CartPoleViewer::new(state_recv);
    app.spin_renderer()
}
