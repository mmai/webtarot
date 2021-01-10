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

- déconnexion partie
- comptue utilisateur activitypub ?
- **UX** indiquer joueur appelé quand il est connu
- scores : indiquer preneur /  partenaire / contrat / echec ? / bonus /
- vérifier cache chargement images 
- **UX** chien : interdire de finir si le chien ne contient pas trois cartes
- **UX** voulez-vous vraiment vous déconnecter ?
- **server** store accounts
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
- **UX** pre-game : nb joueurs connectés
- **server** sécuriser commandes de debug (server_status, debug_ui, etc.)
- **UX** communication -> jitsi
- **UX** annuler dernière carte jouée
- arrêt automatique parties inactives depuis 30mn (sauf si sur pause ?) => sérialisation..
- bug ? affichage enchères en pleine partie


DDD : schema bounded contexts
Gestion Joueurs, parties, historique...
