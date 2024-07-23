#[cfg(target_os = "android")]
use winit::platform::android::activity::AndroidApp;

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(android_app: AndroidApp) {
  use gui::run;
  use winit::event_loop::EventLoop;
  use winit::platform::android::EventLoopBuilderExtAndroid;

  let event_loop = EventLoop::with_user_event()
    .with_android_app(android_app)
    .build()
    .expect("failed to build EventLoop");
  run(event_loop);
}
