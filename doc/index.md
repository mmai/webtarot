Tarot en ligne
==============

`tmuxp load -y webtarot`

## Inspiration

* vocabulaire
  * https://en.wikipedia.org/wiki/Glossary_of_card_game_terms
  * https://en.wikipedia.org/wiki/French_tarot

* calcul scores https://bric-a-brac.org/tarot/scores.php

* cartes
  - https://fr.m.wikipedia.org/wiki/Tarot_nouveau

* design
  * https://play.google.com/store/apps/details?id=com.eryodsoft.android.cards.tarot.lite&hl=fr

## Backlog

- bug ? affichage enchères en pleine partie
- position du bouton finish dog
- déconnexion partie
- **UX** voulez-vous vraiment vous déconnecter ?
- scores : indiquer preneur /  partenaire / contrat / echec ? / bonus /
- **rules**
  - [x] petit au bout
  - chelem (non multiplié par le contrat)
     - [x] annonce éventuelle après écart
     - [x] le preneur commence
     - [x] excuse doit être jouée en dernier, et remporte le plis
     - [x] comptabiliser -> 400 si réussi, -200 si échoué (200 si réussi sans être annoncé)
  - [ ] poignées (non multiplié par le contrat) -> gain au vainqueur de la donne !!
    - la simple poignée : dix atouts (treize atouts à trois joueurs, huit à cinq joueurs) ; la prime est de 20 points ;
    - la double poignée : treize atouts (quinze à trois joueurs, dix à cinq joueurs) ; la prime est de 30 points ;
    - la triple poignée : quinze atouts (dix-huit à trois joueurs, treize à cinq joueurs) ; la prime est de 40 points.
- **rules** optionnelles
  - roi au chien autorisé (montré ensuite) (strictement interdit dans les règles officielles)
  - enchères strictes (pas possibilité de surenchérir après avoir parlé une fois, règles officielles)
  - petit chelem 
    -> 300 points pour un petit chelem annoncé et réalisé, -150 points pour un petit chelem annoncé mais non réalisé
    - tous les plis sauf un (à 4 et 5 joueurs), deux (à 3 joueurs) (variante : trois plis ?)
  - misères (non multiplié par le contrat) -> gain au vainqueur de la donne !!
    - pas d'honneur 10 points,
    - pas d'atout 10 points, 
  - parole
- **UX** indiquer joueur appelé quand il est connu
- **UX** pre-game : nb joueurs connectés
- **server** sécuriser commandes de debug (server_status, debug_ui, etc.)
- **server** store accounts
- **UX** communication -> jitsi
- **UX** annuler dernière carte jouée
- **UX** chien : interdire de finir si le chien ne contient pas trois cartes
- arrêt automatique parties inactives depuis 30mn (sauf si sur pause ?) => sérialisation..


DDD : schema bounded contexts
Gestion Joueurs, parties, historique...
