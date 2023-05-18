use nannou::prelude::*;
use nannou_audio as audio;
use nannou_audio::Buffer;

use std::io::Write;

fn main() {
    nannou::app(model).run()
}

struct Model {
    input_volume: std::sync::mpsc::Receiver<f32>,
    in_stream: audio::Stream<std::sync::mpsc::Sender<f32>>,
}

fn pick_device(host: &audio::Host) -> audio::Device {
    let mut devices: Vec<audio::Device> = host
        .input_devices()
        .expect("Retreive input devices")
        .filter(|device| device.name().is_ok())
        .collect();

    loop {
        print!("Pick an audio device:\n");
        std::io::stdout().flush().unwrap();
        for (i, device) in devices.iter().enumerate() {
            println!("{}: {}", i, device.name().unwrap())
        }

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        if let Ok(index) = input.trim().parse::<usize>() {
            return devices.remove(index);
        }
    }
}

fn model(app: &App) -> Model {
    let audio_host = audio::Host::new();
    let device = pick_device(&audio_host);
    let (volume_send, volume_recieve) = std::sync::mpsc::channel();
    let stream = audio_host
        .new_input_stream(volume_send)
        .capture(move |model: &mut std::sync::mpsc::Sender<f32>, buff: &Buffer<f32>| {
            let n = buff.len();
            if n == 0 {
                return;
            }
            model.send(buff.iter().cloned().sum::<f32>() / (n as f32)).expect("Send input value");
        })
        .channels(1)
        .device(device)
        .build()
        .expect("Create audio stream");

    stream.play().expect("Play input buffer");

    app.new_window()
        .view(view)
        .build()
        .unwrap();


    Model {
        input_volume: volume_recieve,
        in_stream: stream,
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    model.in_stream.play().expect("Play input buffer");

    // Begin drawing
    let win = app.window_rect();
    let t = app.time;
    let draw = app.draw();

    let volume_multiplier = {
        let mut vol = 0.5f32;
        loop {
            match model.input_volume.try_recv() {
                Err(_) => { break; },
                Ok(v) => { vol = (v + 1.0) / 2.0; },
            }
        }
        vol
    };
    println!("Signal received: {volume_multiplier}");

    // Clear the background to blue.
    draw.background().color(BLACK);

    // Create an `ngon` of points.
    let n_points = 5;
    let radius = win.w().min(win.h()) * 0.25 * volume_multiplier;
    let points = (0..n_points).map(|i| {
        let fract = i as f32 / n_points as f32;
        let phase = fract;
        let x = radius * (TAU * phase).cos();
        let y = radius * (TAU * phase).sin();
        pt2(x, y)
    });
    draw.polygon()
        .x(-win.w() * 0.25 * volume_multiplier)
        .color(WHITE)
        .rotate(-t * 0.1)
        .stroke(PINK)
        .stroke_weight(20.0)
        .join_round()
        .points(points);

    // Do the same, but give each point a unique colour.
    let n_points = 7;
    let points_colored = (0..n_points).map(|i| {
        let fract = i as f32 / n_points as f32;
        let phase = fract;
        let x = radius * (TAU * phase).cos();
        let y = radius * (TAU * phase).sin();
        let r = fract;
        let g = 1.0 - fract;
        let b = (0.5 + fract) % 1.0;
        (pt2(x, y), rgb(r, g, b))
    });
    draw.polygon()
        .x(win.w() * 0.25 * volume_multiplier)
        .rotate(t * 0.2)
        .points_colored(points_colored);

    // Write the result of our drawing to the window's frame.
    draw.to_frame(app, &frame).unwrap();
}
