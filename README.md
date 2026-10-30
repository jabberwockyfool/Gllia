# Gllia

Gllia, le launcher des patchs de traduction de la team Saint Eresia, en Rust avec
[egui](https://github.com/emilk/egui) (via `eframe`). Windows uniquement : les
jeux Steam et GOG sont retrouvés grâce au registre.

## Jeux

« Ajouter le jeu » prend le dossier trouvé sur Steam (`steamAppId`) ou GOG
(`gogId`), sinon demande à l'utilisateur de choisir le dossier. « Télécharger
le patch » reste grisé tant que le jeu n'est pas ajouté ; pour l'instant il
ouvre `downloadUrl` dans le navigateur.
