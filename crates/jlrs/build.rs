use julia_metadata::{emit_detected_version_metadata, enable_version_cfgs};

fn main() {
    enable_version_cfgs();
    emit_detected_version_metadata(false);
}
