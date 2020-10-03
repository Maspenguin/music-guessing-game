# &disable_polymer=1
import os
import json
import re
import googleapiclient.discovery
# import googleapiclient.errors

def main():


    api_service_name = "youtube"
    api_version = "v3"
    youtube = googleapiclient.discovery.build(
        api_service_name, api_version, developerKey=os.environ.get("API_KEY"))

    next_page_token = ""
    video_list = []
    while True:
        request = youtube.playlistItems().list(
            part="snippet,contentDetails",
            maxResults=50,
            playlistId="PLdlIrkawA8QY7qAAmOwf9UwL8QTDqzkjq",
            pageToken = next_page_token
        )
        response = request.execute()
        video_list += response["items"]
        if "nextPageToken" in response.keys():
            next_page_token = response["nextPageToken"]
        else:
            break

    game_exp_file = open('game_exp.txt', 'r')
    game_exp_list = game_exp_file.read().splitlines()

    track_list = []
    for video in video_list:
        video_info = video["snippet"]
        video_title = video_info["title"]
        url = "https://youtu.be/"+video_info["resourceId"]["videoId"]
        for game_exp in game_exp_list:
            exp = re.compile(game_exp, re.IGNORECASE)
            s = exp.search(video_title)
            if s:
                track_dict = {"name": s.group('track'), "game": s.group('game'), "url": url}
                track_list.append(track_dict)
                print(track_dict)
                break

    with open('../masbot/tracks.json', 'w') as json_file:
        json.dump(track_list, json_file)

if __name__ == "__main__":
    main()
