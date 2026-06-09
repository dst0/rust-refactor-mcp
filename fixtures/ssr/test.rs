fn main() {
    let x: MyResult<i32> = Ok(10);
    let y: MyResult<i32> = Err("error".to_string());
}
