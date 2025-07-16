# Google Images Scraper en Rust

Un scraper simple en Rust pour effectuer des recherches d’images sur Google et extraire les URLs d’images à partir du HTML de la page de résultats.

---

## Fonctionnalités

- Recherche d’images Google par mots-clés (`query`)
- Extraction des URLs d’images (formats JPG, PNG, GIF, WEBP)
- Filtrage des images indésirables (thumbnails, icônes, logos)
- Gestion automatique d’un User-Agent aléatoire pour limiter les blocages
- Sortie JSON structurée avec métadonnées et résultats
- Utilisation asynchrone avec `tokio` et `reqwest`

---

## Installation

Assurez-vous d’avoir installé Rust et Cargo : https://rustup.rs/

Clonez ce dépôt :

```bash
git clone https://github.com/ton-utilisateur/google-images-scraper-rust.git
cd google-images-scraper-rust
