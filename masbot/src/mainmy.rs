use std::{env, sync::Arc};
use std::sync::Mutex;
use std::collections::HashMap;

use serenity::client::bridge::voice::ClientVoiceManager;

use serenity::{
    model::{channel::Message, gateway::Ready},
    framework::{
        standard::{
            Args, CommandResult,
            macros::{command, group},
        },
    },
    Result as SerenityResult,
    voice::Manager,
    prelude::*,
};

struct VoiceManager;

impl TypeMapKey for VoiceManager {
    type Value = Arc<Mutex<ClientVoiceManager>>;
}

struct Handler {
    state: Mutex<State>
}

struct State {
    map: HashMap<String,String>,
    remember: String
}

impl EventHandler for Handler {
    // Set a handler for the `message` event - so that whenever a new message
    // is received - the closure (or function) passed will be called.
    //
    // Event handlers are dispatched through a threadpool, and so multiple
    // events can be dispatched simultaneously.
    fn message(&self, ctx: Context, msg: Message) {
        //let mut state = self.state.lock().unwrap(); 
        if msg.author.name != "Masbot" {
            let lower = msg.content.trim().to_lowercase();
            let mut tokens: Vec<_> = lower.split_whitespace().map(|x| x.to_string()).collect();
            println!("{:?}",tokens);
            let mut state = self.state.lock().unwrap();
            // substitute variables
            for i in 0..tokens.len() {  
                if let Some(val) = state.map.get(&tokens[i]) {
                    tokens[i] = val.clone();
                }
            }
            for i in 0..tokens.len() {
                if tokens[i] == "=" {
                    if let Some(prev_token) = tokens.get(i-1) {
                        let value_1: String = prev_token.parse().unwrap();
                        if let Some(next_token) = tokens.get(i+1) {
                            let value_2: String = next_token.parse().unwrap();
                            state.map.insert(value_1,value_2);
                            println!("Map: {:?}", state.map);
                        }
                    }
                } 
                if tokens[i] == "+" {
                    if let Some(prev_token) = tokens.get(i-1) {
                        let value_1: i32 = prev_token.parse().unwrap();
                        if let Some(next_token) = tokens.get(i+1) {
                            let value_2: i32 = next_token.parse().unwrap();
                            if let Err(why) = msg.channel_id.say(&ctx.http, value_1+value_2) {
                                println!("Error sending message: {:?}", why);
                            }
                        }
                    }
                }

                if tokens[i] == "*" {
                    if let Some(prev_token) = tokens.get(i-1) {
                        let value_1: i32 = prev_token.parse().unwrap();
                        if let Some(next_token) = tokens.get(i+1) {
                            let value_2: i32 = next_token.parse().unwrap();
                            if let Err(why) = msg.channel_id.say(&ctx.http, value_1*value_2) {
                                println!("Error sending message: {:?}", why);
                            }
                        }
                    }
                }
            }
            if tokens[0] == ".remember" {
                state.remember = tokens[1].to_string();
                println!("{:?}", state.remember);
            }
            if tokens[0] == ".recall" {
                if let Err(why) = msg.channel_id.say(&ctx.http, state.remember.to_string()) {
                    println!("Error sending message: {:?}", why);
                }
            }
            if tokens[0] == ".say" {
                if let Err(why) = msg.channel_id.say(&ctx.http, tokens[1].to_string()) {
                    println!("Error sending message: {:?}", why);
                }

            }
            if tokens[0] == ".join" {
                // join(&mut ctx, &msg);
            }
            if lower.contains("hi masbot") {
                // Sending a message can fail, due to a network error, an
                // authentication error, or lack of permissions to post in the
                // channel, so log to stdout when some error happens, with a
                // description of it.
                if let Err(why) = msg.channel_id.say(&ctx.http, "hello friends im masbot") {
                    println!("Error sending message: {:?}", why);
                }
            }
        }
    }

    // Set a handler to be called on the `ready` event. This is called when a
    // shard is booted, and a READY payload is sent by Discord. This payload
    // contains data like the current user's guild Ids, current user data,
    // private channels, and more.
    //
    // In this case, just print what the current user's username is.
    fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        
        // Client ID is from https://discordapp.com/developers/applications
        // Permissions are generated from the bot section of that same page
        println!("connection url: https://discordapp.com/api/oauth2/authorize?client_id=657021258927439890&scope=bot&permissions=251968");
    }
}

fn main() {
    // //$Env:RUST_LOG = "info"
    //env_logger::init();

    // put this in the terminal:
    // $Env:DISCORD_TOKEN = "thetoken" 
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");
    // Create a new instance of the Client, logging in as a bot. This will
    // automatically prepend your bot token with "Bot ", which is a requirement
    // by Discord for bot users.
    let mut client = Client::new(&token, Handler {state: Mutex::new(State{map: HashMap::new(), remember: "".to_string()})}).expect("Err creating client");


    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start() {
        println!("Client error: {:?}", why);
    }
}
// #[group]
// #[commands(deafen, join, leave, mute, play, ping, undeafen, unmute)]
// struct General;

// #[command]
// fn join(ctx: &mut Context, msg: &Message) -> CommandResult {
//     let guild = match msg.guild(&ctx.cache) {
//         Some(guild) => guild,
//         None => {
//             check_msg(msg.channel_id.say(&ctx.http, "Groups and DMs not supported"));

//             return Ok(());
//         }
//     };

//     let guild_id = guild.read().id;

//     let channel_id = guild
//         .read()
//         .voice_states.get(&msg.author.id)
//         .and_then(|voice_state| voice_state.channel_id);

//     let connect_to = match channel_id {
//         Some(channel) => channel,
//         None => {
//             check_msg(msg.reply(&ctx, "Not in a voice channel"));

//             return Ok(());
//         }
//     };

//     let manager_lock = ctx.data.read().get::<VoiceManager>().cloned().expect("Expected VoiceManager in ShareMap.");
//     let mut manager = manager_lock.lock();

//     if manager.join(guild_id, connect_to).is_some() {
//         check_msg(msg.channel_id.say(&ctx.http, &format!("Joined {}", connect_to.mention())));
//     } else {
//         check_msg(msg.channel_id.say(&ctx.http, "Error joining the channel"));
//     }

//     Ok(())
// }
/// Checks that a message successfully sent; if not, then logs why to stdout.
fn check_msg(result: SerenityResult<Message>) {
    if let Err(why) = result {
        println!("Error sending message: {:?}", why);
    }
}