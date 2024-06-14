mod voxelengine;
use futures::executor::block_on;

fn main() {
    block_on(voxelengine::run());
}
