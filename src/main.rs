mod user;
mod wbi;
use user::User;

#[tokio::main]
async fn main() {
    let user = match User::new().await {
        Ok(u) => u,
        Err(e) => {
            eprintln!("登录失败: {}", e);
            return;
        }
    };

    println!("{:?}", user.cookies);
}
