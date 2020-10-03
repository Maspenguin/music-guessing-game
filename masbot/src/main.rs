use std::{env, sync::Arc};
use std::sync::Mutex;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use serde::{Serialize, Deserialize};

// use rand::Rng;
use rand::seq::SliceRandom;
use serde_json;
use std::char;

use serenity::client::bridge::voice::ClientVoiceManager;

use serenity::{
    model::{channel::Message, gateway::Ready},
    framework::standard::{
        Args, CommandResult,
        macros::{command},
    },
    Result as SerenityResult,
    prelude::*,
    voice
};

// use white_rabbit::{Utc, Scheduler, DateResult, Duration};

struct VoiceManager;

impl TypeMapKey for VoiceManager {
    //type Value = Arc<Mutex<ClientVoiceManager>>;
    type Value = Arc<serenity::prelude::Mutex<ClientVoiceManager>>;
}

struct Handler {
    state: Mutex<State>
}

struct State {
    map: HashMap<String,String>,
    players: HashMap<String, PlayerData>,
    track_map: HashMap<String, Vec<TrackData>>,
    remember: String,
    timer: i32,
    round_track: Option<TrackData>,
    round_game_answer: String,
    round_track_answer: String
}

//#[derive(Serialize)]
#[derive(Serialize, Deserialize, Debug, Clone)]
struct TrackData {
    name: String,
    game: String,
    url: String
}

// #[derive(Deserialize, Debug)]
struct PlayerData {
    last_guess: String,
    score: i32
}

impl EventHandler for Handler {
    // Set a handler for the `message` event - so that whenever a new message
    // is received - the closure (or function) passed will be called.
    //
    // Event handlers are dispatched through a threadpool, and so multiple
    // events can be dispatched simultaneously.
    fn message(&self, mut ctx: Context, msg: Message) {
        //let mut state = self.state.lock().unwrap(); 
        if msg.author.name != "Masbot" {
            let mut state = self.state.lock().unwrap();
            //if the player is not already in the players map, instantiate a new player
            if !state.players.get(&msg.author.name).is_some() {
                let player = PlayerData {
                    last_guess: "".to_string(),
                    score: 0
                };
                state.players.insert(msg.author.name.clone(), player);
            }

            let mut tokens: Vec<String> = msg.content.trim().split_whitespace().map(|x| x.to_string()).collect();
            println!("{:?}",tokens);
            
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
                if let Some(next_token) = tokens.get(1) {
                    state.remember = next_token.to_string();
                }
            }
            if tokens[0] == ".recall" {
                if let Err(why) = msg.channel_id.say(&ctx.http, state.remember.to_string()) {
                    println!("Error sending message: {:?}", why);
                }
            }
            if tokens[0] == ".say" {
                if let Some(next_token) = tokens.get(1) {
                    if let Err(why) = msg.channel_id.say(&ctx.http, next_token.to_string()) {
                        println!("Error sending message: {:?}", why);
                    }
                }
            }
            if tokens[0] == ".join" {
                join(&mut ctx, &msg);
            }
            if tokens[0] == ".play" {
                if let Some(next_token) = tokens.get(1) {
                    play(&mut ctx, &msg, next_token.to_string())
                }                
            }

            if tokens[0] == ".g" {       
                if let Some(next_token) = tokens.get(1) {
                    //(get_mut gets mutable access to the player)
                    state.players.get_mut(&msg.author.name).unwrap().last_guess = next_token.to_string();
                    
                    println!("Player: {:?}", msg.author.name);
                    println!("Game selected: {:?}", state.players.get(&msg.author.name).unwrap().last_guess);
                }                
            }
            // if tokens[0] == ".newplayer" {
            //     let player = PlayerData {
            //         last_guess: "".to_string(),
            //         score: 0
            //     };
            //     state.players.insert(msg.author.name.clone(), player);
            // }
            
            if tokens[0] == ".start" {
                let mut file = File::open("tracks.json").unwrap();
                let mut data = String::new();
                file.read_to_string(&mut data).unwrap();
                // println!("Data: {}",data);
               
                //let track: TrackData =  serde_json::from_reader(file).unwrap();
                let tracks : Vec<TrackData> = serde_json::from_str(&data).unwrap();
                println!("Tracks: {}", tracks.len());

                let mut track_map: HashMap<String, Vec<TrackData>> = HashMap::new();
                //populate track_map based on the list of all tracks (do this once)
                for i in 0..tracks.len() {
                    let game_optional = track_map.get_mut(&tracks.get(i).unwrap().game);
                    match game_optional {
                        Some(game) => game.push(tracks.get(i).unwrap().clone()),
                        None => {
                            let mut new_track_list = Vec::new();
                            new_track_list.push(tracks.get(i).unwrap().clone());
                            track_map.insert(tracks.get(i).unwrap().game.clone(), new_track_list);
                        }
                    }
                }
                // println!("Tracks: {:?}", state.games);
                let track_map_copy = track_map.clone();
                let mut rng = rand::thread_rng();
                //select the game choioces
                let game_vec: Vec<&String> = track_map_copy.keys().collect();
    
                let game_choices: Vec<&&String> = game_vec.choose_multiple(&mut rng, 8).collect();
                println!("Choose: {:?}", game_choices);
                //select the game
                let selected_game = game_choices.choose(&mut rng).clone().unwrap();
                //select the track choices
                let track_vec: Vec<TrackData> = track_map_copy.get(&selected_game.to_string()).unwrap().clone();
                let track_choices: Vec<&TrackData> = track_vec.choose_multiple(&mut rng, 8).collect();
                println!("Choose: {:?}", track_choices);
                //select the track
                let track = track_choices.choose(&mut rng).cloned().cloned().unwrap();
                // let track = state.round_track.as_ref().unwrap();
                let track_name = &track.name;
                println!("Track_name: {}", track_name);
                let track_game = &track.game;
                println!("Track_game: {}", track_game);
                let track_url = &track.url;
                println!("Track_url: {}", track_url);
           
                play(&mut ctx, &msg, track_url.to_string());
                let mut games_message = "Enter a game: \n".to_string();
                let mut letter_val = 65 as u8;
                for game in &game_choices {
                    if game == selected_game {
                        state.round_game_answer = (letter_val as char).to_string();
                        println!("Correct: {}", state.round_game_answer);
                    }
                    games_message.push(letter_val as char);
                    games_message += ": ";
                    games_message += game;
                    games_message += "\n";

                    letter_val += 1;
                }
                if let Err(why) = msg.channel_id.say(&ctx.http, games_message) {
                    println!("Error sending message: {:?}", why);
                }
                state.round_track = Some(track);
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

    
    //let serialized = serde_json::to_string(&point).unwrap();
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
    let mut client = Client::new(&token, Handler {state: Mutex::new(State{map: HashMap::new(), players: HashMap::new(),  track_map: HashMap::new(), remember: "".to_string(), timer: 100, round_track: None, round_game_answer: "".to_string(), round_track_answer: "".to_string()})}).expect("Err creating client");

    {
        let mut data = client.data.write();
        data.insert::<VoiceManager>(Arc::clone(&client.voice_manager));
    }

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start() {
        println!("Client error: {:?}", why);
    }
}

fn join(ctx: &mut Context, msg: &Message) {
    let guild = match msg.guild(&ctx.cache) {
        Some(guild) => guild,
        None => {
            check_msg(msg.channel_id.say(&ctx.http, "Groups and DMs not supported"));

            return;
        }
    };

    let guild_id = guild.read().id;

    let channel_id = guild
        .read()
        .voice_states.get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            check_msg(msg.reply(&ctx, "Not in a voice channel"));

            return;
        }
    };

    let manager_lock = ctx.data.read().get::<VoiceManager>().cloned().expect("Expected VoiceManager in ShareMap.");
    let mut manager = manager_lock.lock();

    if manager.join(guild_id, connect_to).is_some() {
        check_msg(msg.channel_id.say(&ctx.http, &format!("Joined {}", connect_to.mention())));
    } else {
        check_msg(msg.channel_id.say(&ctx.http, "Error joining the channel"));
    }
}

fn play(ctx: &mut Context, msg: &Message, url: String) {

    if !url.starts_with("http") {
        check_msg(msg.channel_id.say(&ctx.http, "Must provide a valid URL"));

        return;
    }

    let guild_id = match ctx.cache.read().guild_channel(msg.channel_id) {
        Some(channel) => channel.read().guild_id,
        None => {
            check_msg(msg.channel_id.say(&ctx.http, "Error finding channel info"));

            return;
        },
    };

    let manager_lock = ctx.data.read().get::<VoiceManager>().cloned().expect("Expected VoiceManager in ShareMap.");
    let mut manager = manager_lock.lock();

    if let Some(handler) = manager.get_mut(guild_id) {
        println!("url: {}",&url);
        let source = match voice::ytdl(&url) {
            Ok(source) => source,
            Err(why) => {
                println!("Err starting source: {:?}", why);

                check_msg(msg.channel_id.say(&ctx.http, "Error sourcing ffmpeg"));

                return;
            },
        };
        handler.stop();
        handler.play(source);

        check_msg(msg.channel_id.say(&ctx.http, "Playing song"));
    } else {
        check_msg(msg.channel_id.say(&ctx.http, "Not in a voice channel to play in"));
    }
}
/// Checks that a message successfully sent; if not, then logs why to stdout.
fn check_msg(result: SerenityResult<Message>) {
    if let Err(why) = result {
        println!("Error sending message: {:?}", why);
    }
}
