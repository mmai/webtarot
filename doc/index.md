Tarot en ligne
==============

`tmuxp load -y webtarot`

## Inspiration

* vocabulaire
  * https://en.wikipedia.org/wiki/Glossary_of_card_game_terms
  * https://en.wikipedia.org/wiki/French_tarot

* cartes
  - https://fr.m.wikipedia.org/wiki/Tarot_nouveau

* design
  * https://play.google.com/store/apps/details?id=com.eryodsoft.android.cards.tarot.lite&hl=fr

## Bugs

- contract failed by 0 (mais points bien comptabilisés..)
- boutons affichés enchères
- points affichés au survol des noms ne correspondent à rien

## Backlog

- arrêt automatique parties inactives depuis 30mn (sauf si sur pause ?) => sérialisation..
- **rules** annonces
  - annulation partie (petit sec)
  - petit au bout
  - poignées
  - misères
  - chelem
- **UX** annuler dernière carte jouée
- **UX** chien : interdire de finir si le chien ne contient pas trois cartes
- **UX** indiquer joueur appelé quand il est connu
- **UX** voulez-vous vraiment vous déconnecter ?
- **server** option désactiver chat cartes jouées
- **UX** pre-game : nb joueurs connectés
- **server** sécuriser commandes de debug (server_status, debug_ui, etc.)
- **design** chien plus large
- **server** store accounts
- **UX** communication -> jitsi


DDD : schema bounded contexts
Gestion Joueurs, parties, historique...
