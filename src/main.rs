mod user;
use user::User;

fn main() {
    let user = match User::new() {
        Ok(u) => u,
        Err(e) => {
            eprintln!("登录失败: {}", e);
            return;
        }
    };
    println!("{:?}", user.cookies);
}
