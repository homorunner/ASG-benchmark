import os
import requests

# Download chess piece images from chess.com and save them locally.
prefixes = [
    ("https://images.chesscomfiles.com/chess-themes/pieces/classic/150", 'classic'),
    ("https://images.chesscomfiles.com/chess-themes/pieces/game_room/150", 'club'),
    ("https://assets-themes.chess.com/image/ejgfv/150", 'neo'),
]

pieces = [
    a+b for a in "wb" for b in "bknqrp"
]

for prefix, name in prefixes:
    folder = os.path.join("images/chess/pieces", name)
    if not os.path.exists(folder):
        os.makedirs(folder)
    for piece in pieces:
        url = f"{prefix}/{piece}.png"
        response = requests.get(url)
        if response.status_code == 200:
            filename = os.path.join(folder, f"{piece}.png")
            with open(filename, "wb") as f:
                f.write(response.content)
            print(f"Downloaded {filename}")
        else:
            print(f"Failed to download {url}")

