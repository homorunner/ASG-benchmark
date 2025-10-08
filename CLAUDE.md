This project is a benchmark that evaluates LLM's performance on abstract strategy game by solving puzzles. The project should be designed to easily handle variant board games like chess, go, xiangqi, quoridor, etc.

Puzzles are defined in text, including board setup, current player, puzzle goal (e.g. find the best move towards win), and expected answer. Game should also be defined in text including game rule, representation of game board and piece moves. Some puzzle may include a series of steps (e.g. a series of single best moves), and LLM can get different score on this puzzle based on how much further it can achieve.

The project use rust as main programming language.
