use steamworks::{Client, UserAchievementStored, UserStatsReceived};

pub(super) fn set_callbacks(client: &Client) {
    // called when you pull down a list of user stats including achievements.
    client.register_callback(|u: UserStatsReceived| {
        if let Err(e) = u.result {
            eprintln!("Steam Error: {:?}", e);
        }
    });

    // called when you push up a state change with an achievement state change
    client.register_callback(|a: UserAchievementStored| {
        println!("Unlocked Achievement: {}", &a.achievement_name);
    });
}
