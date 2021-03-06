use std::{env};
use std::sync::Mutex;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use serde::{Serialize, Deserialize};

// use rand::Rng;
use rand::seq::SliceRandom;
use serde_json;
use std::char;
use std::time::Duration;
use std::time::Instant;
// use std::collections::{HashSet,env};
use async_std::task;
// futures = "0.3.6"

use serenity::{
    async_trait,
    // collector::MessageCollectorBuilder,
    model::{channel::Message, gateway::Ready},
    Result as SerenityResult,
    prelude::*,
};

use songbird::SerenityInit;
// use white_rabbit::{Utc, Scheduler, DateResult, Duration};

struct Handler {
    state: Mutex<State>
}

struct State {
    map: HashMap<String,String>,
    players: HashMap<String, PlayerData>,
    track_map: HashMap<String, Vec<TrackData>>,
    // round_track: Option<TrackData>,
    // round_game_answer: String,//Letter
    // round_track_answer: String,//Letter
    round_track_message: String,
    max_timer_duration: i32,
    auto_next: bool
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
    user: serenity::model::user::User,
    game_guess: String,
    track_guess: String,
    score: i32
}

struct DirectMessage {
    user: serenity::model::user::User,
    message: String,
}

#[async_trait]
impl EventHandler for Handler {
    // Set a handler for the `message` event - so that whenever a new message
    // is received - the closure (or function) passed will be called.
    //
    // Event handlers are dispatched through a threadpool, and so multiple
    // events can be dispatched simultaneously.
    async fn message(&self, mut ctx: Context, msg: Message) {
        if msg.author.name != "Masbot" {
            let mut tokens: Vec<String> = msg.content.trim().split_whitespace().map(|x| x.to_string()).collect();
            println!("{:?}",tokens);
            
            // substitute variables
            {
                let state = self.state.lock().unwrap();
                for i in 0..tokens.len() {  
                    if let Some(val) = state.map.get(&tokens[i]) {
                        tokens[i] = val.clone();
                        
                    }
                }
            }
            match msg.guild(&ctx.cache).await {
                Some(_guild) => {
                    if tokens[0] == ".join" {
                        join(&mut ctx, &msg).await;
                    }
                    if tokens[0] == ".play" {
                        if let Some(next_token) = tokens.get(1) {
                            play(&mut ctx, &msg, next_token.to_string()).await;
                        }                
                    }
                    if tokens[0] == ".timer" {
                        if let Some(next_token) = tokens.get(1) {
                            if let Ok(new_timer_duration) = next_token.parse::<i32>() {
                                {
                                    let mut state = self.state.lock().unwrap();
                                    state.max_timer_duration = new_timer_duration;
                                }
                                if let Err(why) = msg.channel_id.say(&ctx.http, "Timer duration set to ".to_string() + &new_timer_duration.to_string()).await {
                                    println!("Error sending message: {:?}", why);
                                }
                            }
                        }
                    }
                    if tokens[0] == ".autonext" {
                        let auto_next = {
                            let mut state = self.state.lock().unwrap();
                            state.auto_next = !state.auto_next;
                            state.auto_next
                        };
                        {
                            let mut state = self.state.lock().unwrap();                            
                        }
                        if auto_next == true {
                            if let Err(why) = msg.channel_id.say(&ctx.http, "Auto next is now on".to_string()).await {
                                println!("Error sending message: {:?}", why);
                            }
                        }
                        else {
                            if let Err(why) = msg.channel_id.say(&ctx.http, "Auto next is now off".to_string()).await {
                                println!("Error sending message: {:?}", why);
                            }
                        }
                  
                    }
                    if tokens[0] == ".start" || tokens[0] == ".s" || tokens[0] == ".next" || tokens[0] == ".n" {
                        let mut auto_next = {
                            let state = self.state.lock().unwrap();
                            state.auto_next
                        };
                        if auto_next {
                            while auto_next {
                                play_round(self, &mut ctx, &msg).await;
                                auto_next = {
                                    let state = self.state.lock().unwrap();
                                    state.auto_next
                                };
                            }
                        }
                        else {
                            play_round(self, &mut ctx, &msg).await;
                        }
                    }
                    if tokens[0] == ".signin" || tokens[0] == ".si"{  
                        //if the player is not already in the players map, instantiate a new player
                        {
                            let mut state = self.state.lock().unwrap();
                            if !state.players.get(&msg.author.name).is_some() {
                                let player = PlayerData {
                                    user: msg.author.clone(),
                                    game_guess: "".to_string(),
                                    track_guess: "".to_string(),
                                    score: 0
                                };
                                state.players.insert(msg.author.name.clone(), player);
                            }
                        }
                        if let Err(why) = &msg.author.direct_message(&ctx, |m| { m.content("Welcome!") }).await {
                            println!("Error sending message: {:?}", why);
                        }
                    }
                    if tokens[0] == ".signoff" || tokens[0] == ".so" {  
                        {
                            let mut state = self.state.lock().unwrap();
                            state.players.remove(&msg.author.name);
                        }
                        if let Err(why) = &msg.author.direct_message(&ctx, |m| { m.content("Good bye.") }).await {
                            println!("Error sending message: {:?}", why);
                        }
                    }
                },
                None => {
                    //Message was not post in a server i.e. it is from a direct message
                    if tokens.len() == 1 && tokens.get(0).unwrap().len() == 1 {
                        let round_track_message_option = {//if a game answer is selected this is given a value so that the track options are given to the player.
                            let mut state = self.state.lock().unwrap();
                            if state.players.get(&msg.author.name).unwrap().game_guess == "?" {//i.e. if game guess has not been selected yet
                                //(get_mut gets mutable access to the player)
                                if let Some(author) = state.players.get_mut(&msg.author.name) {
                                    author.game_guess = tokens.get(0).unwrap().to_string().to_uppercase();
                                }
                                else {
                                    println!("Unregistered player: {:?}", msg.author.name);
                                }
                                Some(state.round_track_message.clone())
                            }
                            else {
                                //(get_mut gets mutable access to the player)
                                if let Some(author) = state.players.get_mut(&msg.author.name) {
                                    author.track_guess = tokens.get(0).unwrap().to_string().to_uppercase();
                                }
                                else {
                                    println!("Unregistered player: {:?}", msg.author.name);
                                }
                                None
                            }
                        };
                        
                        //send track options to the player
                        if let Some(round_track_message) = round_track_message_option {
                            if let Err(why) = &msg.author.direct_message(&ctx, |m| m.content(round_track_message)).await {
                                println!("Error sending message: {:?}", why);
                            }
                        }
                    }
                    else {
                        if let Err(why) = &msg.author.direct_message(&ctx, |m| m.content("Please only enter a single letter as your answer.")).await {
                            println!("Error sending message: {:?}", why);
                        }
                    }
                }
            };
        }
    }

    // Set a handler to be called on the `ready` event. This is called when a
    // shard is booted, and a READY payload is sent by Discord. This payload
    // contains data like the current user's guild Ids, current user data,
    // private channels, and more.
    //
    // In this case, just print what the current user's username is.
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        // Client ID is from https://discordapp.com/developers/applications
        // Permissions are generated from the bot section of that same page
        println!("connection url: https://discordapp.com/api/oauth2/authorize?client_id=657021258927439890&scope=bot&permissions=251968");
    }
}

//Sends each message to its specified player
async fn send_direct_messages(ctx: &Context, direct_messages: &[DirectMessage]) -> Vec<Message> {
    let mut sent_messages = Vec::new();
    for direct_message in direct_messages {
        if let Ok(message) = direct_message.user.direct_message(&ctx, |m| { m.content(&direct_message.message) }).await {
            sent_messages.push(message);
        }
    }
    return sent_messages;
}


//Sends a particular message to each player
async fn send_direct_message_to_all(handler: &Handler, ctx: &Context, message: String) -> Vec<Message> {
    let mut direct_messages = vec!();
    {
        let mut state = handler.state.lock().unwrap();
        let players = &mut state.players;
        for (_, player_details) in players.iter() {
            direct_messages.push(DirectMessage {
                user: player_details.user.clone(),
                message: message.to_string()
            });
        }
    }
    return send_direct_messages(&ctx, &direct_messages).await;
}

async fn play_round(handler: &Handler, ctx: &mut Context, msg: &Message) {
    let track_map_copy = get_track_map_copy(handler);
                 
    //select the game choices
    let game_vec: Vec<&String> = track_map_copy.keys().collect();
    let game_choices: Vec<&&String> = game_vec.choose_multiple(&mut rand::thread_rng(), 8).collect();
    println!("Choose: {:?}", game_choices);
    //select the game
    let selected_game = game_choices.choose(&mut rand::thread_rng()).clone().unwrap();
    
    //reset player answers
    {
        let mut state = handler.state.lock().unwrap();
        let players = &mut state.players;
        for (_, player_details) in players.iter_mut() {
            player_details.track_guess = "?".to_string();
            player_details.game_guess = "?".to_string();
            println!("After reset: {:?}", player_details.track_guess);
            println!("After reset: {:?}", player_details.game_guess);
        }
    }

    //select the track choices
    let track_vec: Vec<TrackData> = track_map_copy.get(&selected_game.to_string()).unwrap().clone();
    let track_choices: Vec<&TrackData> = track_vec.choose_multiple(&mut rand::thread_rng(), 8).collect();
    println!("Choose: {:?}", track_choices);
    //select the track
    let selected_track = track_choices.choose(&mut rand::thread_rng()).cloned().cloned().unwrap();
    println!("Track_name: {}", &selected_track.name);
    println!("Track_game: {}", &selected_track.game);
    println!("Track_url: {}", &selected_track.url);
    play(ctx, &msg, selected_track.url.to_string()).await;

    //Construct game name question message
    let mut letter_number = 0;
    let mut game_answer = "?".to_string();//(The letter allias)
    // let mut all_sent_games_messages = vec!(); 
    let mut game_message = "Select a game name: \n".to_string();
    for game in &game_choices {
        if game == selected_game {
            game_answer = (((65+letter_number)as u8) as char).to_string();
            println!("Game answer: {}", game_answer);
        }
        game_message.push(((65+letter_number) as u8) as char);
        game_message += ": ";
        game_message += game;
        game_message += "\n";
        letter_number += 1;
    }
    send_direct_message_to_all(handler, &ctx, game_message).await;

    //Construct track name question message
    let mut track_message = "Select a track title: \n".to_string();
    let mut letter_number = 0;
    // let mut letter_val = 65 as u8;
    let mut track_answer = "?".to_string(); //(the letter alias)
    for track in &track_choices {
        if track.name == selected_track.name {
            track_answer = (((65+letter_number)as u8) as char).to_string();
            println!("Track answer: {}", track_answer);
        }
        track_message.push(((65+letter_number) as u8) as char);
        //track_message += format!(": {}\n", track.name);
        track_message += ": ";
        track_message += &track.name;
        track_message += "\n";

        letter_number += 1;
    }
    //let sent_track_messages = ...

    //Store track name question message for later 
    {
        let mut state = handler.state.lock().unwrap();
        state.round_track_message = track_message;
    }
    
    //Send initial time left messages to each player
    
    let timer_duration = { 
        let state = handler.state.lock().unwrap();
        state.max_timer_duration
    };
    let mut timer_messages = send_direct_message_to_all(handler, &ctx, "Time left: ".to_string() + &timer_duration.to_string()).await;
    let start_time = Instant::now();
    //Periodically update the time left messages sent to each player
    while start_time.elapsed().as_secs_f32() < timer_duration as f32 {

        task::sleep(Duration::from_secs(1)).await;
        for message in &mut timer_messages {
            let time_left = (timer_duration as f32 - start_time.elapsed().as_secs_f32()).round();
            message.edit(&*ctx, |m| { m.content("Time left: ".to_string() + &time_left.to_string()) }).await.unwrap();
        }
    }
    
    //Times up, send results
    send_direct_message_to_all(handler, &ctx, "Times up!".to_string()).await;
    send_direct_message_to_all(handler, &ctx, "Correct game answer was: ".to_string() + selected_game).await;
    send_direct_message_to_all(handler, &ctx, "Correct track answer was: ".to_string() + &selected_track.name).await;
    let mut scoreboard_message = "Scores:".to_string();  
    {
        let mut state = handler.state.lock().unwrap();
        let players = &mut state.players;
        scoreboard_message += "\n";
        for (player_name, player_details) in players.iter_mut() {
            println!("Player name: {:?}", player_name);
            println!("Players Game guess: {:?}", player_details.game_guess);
            println!("Players Track guess: {:?}", player_details.track_guess);
            let mut round_score = 0;
            if player_details.game_guess == game_answer {
                player_details.score += 1;
                round_score += 1;
            }
            if player_details.track_guess == track_answer {
                player_details.score += 1;
                round_score += 1;
            }
            scoreboard_message += &player_name.to_string();
            scoreboard_message += ": ";
            scoreboard_message += &player_details.score.to_string();
            scoreboard_message += " (+";
            scoreboard_message += &round_score.to_string();
            scoreboard_message += ")";
            scoreboard_message += "\n";
        }
    }
    if let Err(why) = msg.channel_id.say(&ctx.http, scoreboard_message).await {
        println!("Error sending message: {:?}", why);
    }
}

#[tokio::main]
async fn main() {
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
    let mut client = Client::builder(&token)
    .event_handler(Handler {
        state: Mutex::new(
            State{
                map: HashMap::new(), 
                players: HashMap::new(),  
                track_map: HashMap::new(),
                round_track_message: "".to_string(),
                max_timer_duration: 45,
                auto_next: true
            }
        )
    })
    .register_songbird()
    .await
    .expect("Err creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

async fn join(ctx: &mut Context, msg: &Message) {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let channel_id = guild
        .voice_states.get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            check_msg(msg.channel_id.say(&ctx.http, "Not in a voice channel").await);
            return;
        }
    };


    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();

    let _handler = manager.join(guild_id, connect_to).await;
}

async fn play(ctx: &mut Context, msg: &Message, url: String) {
    if !url.starts_with("http") {
        check_msg(msg.channel_id.say(&ctx.http, "Must provide a valid URL").await);

        return;
    }

    let guild = match msg.guild(&ctx.cache).await {
        Some(guild) => guild,
        None => {
            check_msg(msg.channel_id.say(&ctx.http, "Message not from a server.").await);
            return;
        }
    };
    let guild_id = guild.id;


    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        let source = match songbird::ytdl(&url).await {
            Ok(source) => source,
            Err(why) => {
                println!("Err starting source: {:?}", why);

                check_msg(msg.channel_id.say(&ctx.http, "Error sourcing ffmpeg").await);

                return;
            },
        };
        let _track_handle = handler.play_only_source(source);
        check_msg(msg.channel_id.say(&ctx.http, "Playing song").await);
    } else {
        check_msg(msg.channel_id.say(&ctx.http, "Not in a voice channel to play in").await);
    }
}

/// Checks that a message successfully sent; if not, then logs why to stdout.
fn check_msg(result: SerenityResult<Message>) {
    if let Err(why) = result {
        println!("Error sending message: {:?}", why);
    }
}

fn get_track_map_copy(handler: &Handler) -> HashMap<String, Vec<TrackData>> {
    let mut state = handler.state.lock().unwrap();
    if state.track_map.is_empty() {
        let mut file = File::open("tracks.json").unwrap();
        let mut data = String::new();
        file.read_to_string(&mut data).unwrap();
        
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
        println!("New track_map");
        state.track_map = track_map.clone();
    }
    return state.track_map.clone();
}
