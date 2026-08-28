PraiseChords (Rust) - esqueleto

Passos rápidos:

1. Copiar `tailwind.css` gerado do projeto Next.js para `static/tailwind.css`.
2. Copiar `htmx.min.js` e `alpine.min.js` para `static/` ou usar CDN.
3. Criar `.env` baseado em `.env.example` com `DATABASE_URL` apontando para a mesma base de dados PostgreSQL.
4. Build e run:

```bash
# instalar Rust toolchain
cargo build
cargo run
```

Servidor inicia em `http://127.0.0.1:8080` e expõe `/api/songs` e `/static/*`.
