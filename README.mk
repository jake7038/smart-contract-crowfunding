# Crowdfunding em CosmWasm

Contrato inteligente de **crowdfunding com meta e reembolso**, escrito em Rust para a plataforma [CosmWasm](https://cosmwasm.com). Projeto desenvolvido para a disciplina de **Arquitetura de Linguagens de Programação**.

## Visão geral

O criador da campanha define uma **meta**, uma **moeda** e uma **duração**. Qualquer pessoa pode contribuir até o prazo final. Depois disso:

- **Meta atingida:** o dono da campanha saca todo o valor arrecadado.
- **Meta não atingida:** cada contribuinte pode pedir o reembolso do que enviou.

```
            ┌──────────────┐   prazo termina   ┌──────────────┐
 Contribute │     Open     │ ────────────────► │  total ≥ meta │──► Withdraw (dono)
 ──────────►│              │                   ├──────────────┤
            └──────────────┘                   │  total < meta │──► Refund (cada contribuinte)
                                               └──────────────┘
```

## Estrutura do projeto

```
crowdfunding/
├── .cargo/config.toml   # flags de build para Wasm e alias `cargo wasm`
├── Cargo.toml
└── src/
    ├── lib.rs           # declaração dos módulos
    ├── contract.rs      # instantiate, execute, query e testes
    ├── msg.rs           # mensagens do contrato (enums)
    ├── state.rs         # dados persistidos
    └── error.rs         # erros tipados
```

## Requisitos

- [Rust](https://rustup.rs) (toolchain estável)
- Target WebAssembly:

```bash
rustup target add wasm32-unknown-unknown
```

## Como usar

### Rodar os testes

```bash
cargo test
```

Os testes são unitários e não exigem nó, carteira nem rede: usam os mocks de `cosmwasm_std::testing`.

### Compilar para WebAssembly

Crie o arquivo `.cargo/config.toml` com o conteúdo abaixo (necessário em versões recentes do Rust, porque as funções fornecidas pela blockchain, como `db_read` e `db_write`, só existem em tempo de execução):

```toml
[target.wasm32-unknown-unknown]
rustflags = ["-C", "link-arg=--allow-undefined"]

[alias]
wasm = "build --release --lib --target wasm32-unknown-unknown"
```

Depois:

```bash
cargo wasm
```

O arquivo gerado fica em `target/wasm32-unknown-unknown/release/crowdfunding.wasm`.

## Interface do contrato

### Instanciação

```json
{
  "goal": "100",
  "denom": "untrn",
  "duration_seconds": 1000
}
```

Quem instancia o contrato se torna o **dono** da campanha. O prazo é calculado a partir do horário do bloco de instanciação.

### Execução

| Mensagem | Quem pode | Quando | Efeito |
|---|---|---|---|
| `{"contribute": {}}` | Qualquer pessoa | Antes do prazo | Registra a contribuição enviada junto com a mensagem |
| `{"withdraw": {}}` | Apenas o dono | Após o prazo, com meta atingida | Envia o total arrecadado ao dono |
| `{"refund": {}}` | Qualquer contribuinte | Após o prazo, com meta **não** atingida | Devolve o valor contribuído por quem chamou |

Na `contribute`, os fundos devem ser enviados na moeda da campanha (`denom`) e com valor maior que zero.

### Consultas

| Consulta | Retorno |
|---|---|
| `{"get_config": {}}` | Dono, meta, moeda e prazo |
| `{"get_status": {}}` | `status` (`open`, `succeeded` ou `failed`) e `total` arrecadado |
| `{"get_contribution": {"address": "..."}}` | Valor contribuído pelo endereço |

## Erros

| Erro | Significado |
|---|---|
| `Unauthorized` | Ação restrita ao dono |
| `CampaignEnded` | Tentativa de contribuir após o prazo |
| `CampaignOpen` | Saque ou reembolso antes do prazo |
| `GoalNotReached` | Saque com meta não atingida |
| `GoalReached` | Reembolso com meta atingida |
| `NothingToRefund` | O endereço não tem saldo a reembolsar |
| `AlreadyWithdrawn` | Os fundos já foram sacados |
| `Payment` | Moeda incorreta, nenhuma moeda ou valor zero na contribuição |

## Decisões de projeto e conceitos de linguagem

- **Enums e pattern matching:** `ExecuteMsg`, `QueryMsg` e `Status` modelam o domínio, e o `match` obriga a tratar todos os casos.
- **Tratamento de erros sem exceções:** funções retornam `Result<T, ContractError>` e propagam falhas com o operador `?`. O `ContractError` usa `thiserror`.
- **Ownership e borrowing:** o estado é acessado por referências (`&info.sender`, `deps.storage`), sem cópias desnecessárias.
- **Sistema de tipos como defesa:** `Uint128` evita overflow silencioso, e `Addr` diferencia um endereço validado de uma `String` qualquer.
- **Segurança por ordem das operações:** no reembolso, o saldo do contribuinte é zerado **antes** de gerar a mensagem de envio, evitando reembolso duplo. O saque é protegido pela flag `WITHDRAWN`.
- **Macros e metaprogramação:** `#[cw_serde]` e `#[entry_point]` geram código de serialização e os pontos de entrada exigidos pelo Wasm.

## Limitações

- Existe apenas uma campanha por instância do contrato.
- Não há cancelamento antecipado pelo dono.
- Não há deploy configurado em testnet; o projeto é validado por testes unitários.

## Possíveis extensões

- Testes de integração com `cw-multi-test`, simulando saldos reais de banco
- Valor mínimo ou máximo por contribuição
- Mensagem `Cancel` com reembolso geral
- Deploy em uma testnet de CosmWasm

## Autor

Rafael: estudante de Engenharia de Software no CEFET/RJ.