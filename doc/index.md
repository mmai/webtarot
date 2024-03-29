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

- [ ] bug : archives en lignes non actives
- [ ] bug : bot le partenaire a joué le petit, le bot a joué le deux au lieu du 21
- **server** enregistrement des parties
  - enregistrer les parties lorsqu'elles sont terminées 
  - listing
  - permet interruption et reprise serveur ?
  - remplacer completement btreemap par sled ?

- scores : indiquer preneur /  partenaire / contrat / echec ? / bonus /

- **UX** volet résultats
- **bot** envoyer des messages au chat (selon résultat du trick, différent selon n° de bot) 
- bug : son cartes jouées

- **server** store accounts
- vérifier random
- docker : étapes supplémentaires pour publier : exécuter localement ?
- **refacto** retirer la dépendence Universe de Game : possible ? cf. https://docs.google.com/presentation/d/1ov5957xmm8s9V2F32AgXbaaQL0nCPai58PavU6jn0jA/edit#slide=id.p 
- compte utilisateur activitypub https://socialhub.activitypub.rocks/t/single-sign-on-for-fediverse/712
- **UX** chien : interdire de finir si le chien ne contient pas trois cartes
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
- **server** sécuriser commandes de debug (server_status, debug_ui, etc.)
- **UX** communication -> jitsi
- **UX** annuler dernière carte jouée
- arrêt automatique parties inactives depuis 30mn (sauf si sur pause ?) => sérialisation..

DDD : schema bounded contexts
Gestion Joueurs, parties, historique...

## Backlog perfs

### peu d'impact

- invitations bots 
  - remplacer unix file socket par un canal
  - garder un canal ouvert au lieu de le recréer à chaque fois (webgame_server > universe.rs)


