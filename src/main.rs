mod download;
mod user;
mod video;
mod wbi;
use user::User;

use crate::video::VideoBasicInfo;

#[tokio::main]
async fn main() {
    let user = match User::new_from_file("~/.config/ov-bilidown/cookies.txt") {
        Ok(u) => u,
        Err(e) => match User::new().await {
            Ok(u) => {
                u.save_to_file("~/.config/ov-bilidown/cookies.txt").unwrap();
                u
            }
            Err(e) => {
                eprintln!("登录失败: {}", e);
                return;
            }
        },
    };

    let bvid = "BV1NfxMedEU6";

    let video = VideoBasicInfo::new_from_bvid(&user, bvid).await.unwrap();

    video
        .download_best_quality_audios_to_file(&user, "./downloads")
        .await
        .unwrap();

    println!("{:?}", video);
}
