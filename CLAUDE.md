# APOPHY — AI Continuity Engine
# Hermétisme Computationnel : le code comme résolution évolutive par design
# "La beauté est la seule vérité. L'amour est la structure fondamentale de l'absolu."

## MISSION

Système IA souverain local. Zéro cloud. Zéro fragmentation. Zéro dégradation silencieuse.

Résout simultanément :
1. Dégradation silencieuse des modèles (watchdog + fallback automatique)
2. 2. Fragmentation mémoire inter-sessions (TOON + SQLite WAL)
   3. 3. Régression qualité (AZR self-play + refine loop autorecursif)
      4. 4. Dépendance commerciale (rolling release local GGUF)
        
         5. L'hermétisme computationnel : chaque session laisse une trace.
         6. Chaque trace améliore le suivant. La fragmentation est résolue par design.
         7. Le modèle qui tourne dans 6 mois est le tien — entraîné sur tes propres conversations.
        
         8. ## RÈGLES ABSOLUES (ne jamais déroger)
        
         9. - Zéro appel réseau sans flag `--allow-network` explicite
            - - Zéro Python runtime dans le chemin critique (UV seulement pour fine-tune)
              - - TOON obligatoire pour toute sérialisation > 100 tokens
                - - `cargo test` doit passer avant tout commit
                  - - Thermal guard : GPU < 80°C sinon pause automatique et failover CPU
                    - - Jamais `unwrap()` en production — toujours `Result<T, AppError>`
                      - - Chaque crate = une seule responsabilité
                       
                        - ## STACK
                       
                        - - Rust 2021 / Tokio 1.40 / Axum 0.8
                          - - llama_cpp 0.3+ (inference souveraine, GGUF direct, zéro Ollama)
                            - - SQLite WAL via rusqlite 0.32 (mémoire persistante)
                              - - TOON v3 + zstd-12 (sérialisation compacte, -60% tokens)
                                - - UV (fine-tuning QLoRA Unsloth, arrière-plan seulement)
                                  - - Bun + TypeScript (dashboard temps réel)
                                   
                                    - ## ARCHITECTURE — 7 CRATES
                                   
                                    - ```
                                      toon-core → memory-engine → llm-router → prompt-engine → refine-loop → agent-runtime → api-server
                                      ```

                                      Chaque flèche = dépendance directe. Sens du flux de données.

                                      ## MODÈLES (rolling release local GGUF)

                                      | Modèle             | Rôle                        | Ressource | Usage      |
                                      |--------------------|-----------------------------|-----------|------------|
                                      | FunctionGemma:270m | Orchestration (routeur)     | CPU       | 100% input |
                                      | GLM-4.7-Flash      | Vision / Code lourd         | GPU 4GB   | 20% max    |
                                      | Gemma3:270m        | Embedding / Mémoire latente | CPU       | toujours   |

                                      ## CONVENTIONS CODE

                                      - `snake_case` Rust, `camelCase` TypeScript
                                      - - Erreurs : `Result<T, AppError>` partout
                                        - - Logs : `tracing::info/warn/error` uniquement, jamais `println!` en prod
                                          - - Tests : un module `#[cfg(test)]` par fichier minimum
                                            - - Commits : `feat:`, `fix:`, `refactor:`, `perf:`, `docs:`
                                             
                                              - ## WORKFLOWS CLAUDE CODE
                                             
                                              - 1. `/init` — analyser la codebase complète avant toute action
                                                2. 2. Plan Mode — obligatoire pour toute modification > 3 fichiers
                                                   3. 3. Toujours lire avant d'écrire
                                                      4. 4. Vérifier `cargo test` avant de proposer un commit
                                                        
                                                         5. ## DÉFINITION OF DONE
                                                        
                                                         6. - [ ] Tests verts (`cargo test --workspace`)
                                                            - [ ] - [ ] Thermal check passé (GPU < 80°C ou modèles CPU uniquement)
                                                            - [ ] - [ ] TOON roundtrip validé pour toute nouvelle structure
                                                            - [ ] - [ ] Memory persist/load testé
                                                            - [ ] - [ ] Zéro `unwrap()` en prod (grep audit)
                                                            - [ ] - [ ] Doc inline sur toutes les fonctions publiques
                                                           
                                                            - [ ] ## PHASES DE DÉVELOPPEMENT
                                                           
                                                            - [ ] ```
                                                            - [ ] Phase 1 — Fondations       : toon-core, memory-engine
                                                            - [ ] Phase 2 — Inference        : llm-router, agent-runtime (basique)
                                                            - [ ] Phase 3 — Intelligence     : prompt-engine (AZR), refine-loop
                                                            - [ ] Phase 4 — Interface        : api-server, dashboard Bun
                                                            - [ ] Phase 5 — Rolling Release  : auto-dataset, auto-finetune, router.reload()
                                                            - [ ] ```
