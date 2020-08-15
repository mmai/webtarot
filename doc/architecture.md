# Architecture

## Protocol

=> utilisé par _Server_ et _Client_

* lib : wrapper autour des autres fichiers
* game :
  - étapes du jeu
  - structs d'état du jeu et des joueurs
* message :
  - command : actions des joueurs (dont actions de connexion/déconnexion à la session)
  - messages : messages du moteur
* player :
  - struct joueur pour la connexion

## Server

* main : wrapper
* utils : generate join code
* server : communication client <-> universe & game
* universe : gestion lobby (connexions, creation game)
* game : etat du jeu (utilise universe)
* board : construit le plateau initial, ...?

## Client

* lib : wrapper
* utils : format join code
* api : communication client (app) <-> server
* app : point d'entrée, sélectionne page (composant Yew )
* /components : composants custom Yew
* /views :  pages (composants Yew)


