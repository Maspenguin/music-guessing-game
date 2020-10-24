# Music Guessing Game
This is a music guessing game it is run for multiple players by a bot using the chat service Discord.
It was designed with video game music and soundtracks in mind.
## JSON Writer
The JSON writer is used to generate a JSON file for a particular youtube playlist. The JSON file will then contain an entry for every track with the soundtrack name and the video URL.
Before running the script the regular expressions in game_exp.txt will need to be setup so that each soundtrack is caught. Usually, only one expression will be needed for each soundtrack.
If you want to use this script yourself, you will also need to setup your own API key for the Youtube Data API
## Masbot 
When the bot is running it will respond to certain commands in the chat.

".signin" will list the user as a player, they will be send direct messages from the bot allowing the player to submit their answer.

".signoff" will remove the user as a player in the game.

".join" will make the bot join the voice channel that the user is presently in.

".start" or ".next" will start the next round, the bot will select and play a track through the voice channel. The players then have to guess what it is.

".g [answer]" will submit [answer] as the players guess for the game name.
".t [answer]" will submit [answer] as the players guess for the track name.
The players should submit the letter corresponding to the correct answer.
Players should submit the game name first, so they are given choices for the track name.