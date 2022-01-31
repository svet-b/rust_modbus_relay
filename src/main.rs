use std::{env, thread, time};

const DFLT_TTY_PATH: &str = "/dev/rs485";
const TTY_PATH_ENV: &str = "TTY_PATH";
const DFLT_BAUD: u32 = 9600;
const DFLT_SLAVE_ID: u8 = 255;
const DFLT_RELAY_NUM: u8 = 0;
const DFLT_DURATION: u16 = 10;

async fn set_state(
    ctx: &mut tokio_modbus::client::Context,
    relay_num: u8,
    state: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use tokio_modbus::client::Writer;
    ctx.write_single_coil(relay_num.into(), state).await?;
    Ok(())
}

async fn get_state(
    ctx: &mut tokio_modbus::client::Context,
    relay_num: u8,
) -> Result<bool, Box<dyn std::error::Error>> {
    use tokio_modbus::client::Reader;
    let rsp = ctx.read_coils(0, 8).await?;
    Ok(rsp[relay_num as usize])
}

async fn do_loop(
    ctx: &mut tokio_modbus::client::Context,
    relay_num: u8,
    duration: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Reading relay state");
    let state = get_state(ctx, relay_num).await?;
    println!("Relay state is: {state}");

    println!("Setting relay {relay_num} to ON");
    set_state(ctx, relay_num, true).await?;

    println!("Reading relay {relay_num} state");
    let state = get_state(ctx, relay_num).await?;
    println!("Relay state is: {state}");

    println!("Waiting for {duration} seconds");
    let sleep_time = time::Duration::from_secs(duration.into());
    thread::sleep(sleep_time);

    println!("Setting relay {relay_num} to OFF");
    set_state(ctx, relay_num, false).await?;

    println!("Reading relay {relay_num} state");
    let state = get_state(ctx, relay_num).await?;
    println!("Relay state is: {state}");

    Ok(())
}

#[tokio::main(flavor = "current_thread")]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Process arguments
    // Argument 1 - relay number
    // Argument 2 - time to turn on for (seconds)
    let relay_num = match env::args().nth(1) {
        Some(v) => v.parse::<u8>().unwrap(),
        None => DFLT_RELAY_NUM,
    };
    let duration = match env::args().nth(2) {
        Some(v) => v.parse::<u16>().unwrap(),
        None => DFLT_DURATION,
    };
    println!("Turning on relay {} for {} seconds", relay_num, duration);

    use tokio_modbus::client::rtu;
    use tokio_modbus::slave::Slave;
    use tokio_serial::SerialStream;

    let tty_path = env::var(TTY_PATH_ENV).unwrap_or(DFLT_TTY_PATH.to_string());
    let slave = Slave(DFLT_SLAVE_ID);
    let baud_rate = DFLT_BAUD;

    let builder = tokio_serial::new(tty_path, baud_rate);
    let port = SerialStream::open(&builder).unwrap();
    let mut ctx = rtu::connect_slave(port, slave).await?;

    do_loop(&mut ctx, relay_num, duration).await?;

    Ok(())
}
