# Achievements

> [Home](Home) · [Stores](Stores-and-Saves) · **Achievements** ·
> [Settings](Settings) · [Troubleshooting](Troubleshooting)

Achievements are shown on the game page, with icons, descriptions, unlock
dates, rarity and progress when the store publishes them.

## Epic achievements

Epic uses Legendary:

```bash
legendary achievements <app> --json
```

The parser accepts JSON surrounded by log lines, a null list and the older text
table (`Total achievements`, `Completed`, `In progress`). If the metadata cache
is incomplete, Quark runs `legendary info` and retries once. A missing session
or a game without achievements produces an explicit message instead of an
unreadable error.

## GOG achievements

GOG queries `gameplay.gog.com` with the gogdl session. Both sources are
normalized to the same rows: icon, name, description, unlock date, rarity and
progress. Icons are cached locally.

An achievement unlocked during the session can carry a **session** badge.

## Local achievements API

A game or script can report an unlock through the small local server:

```bash
curl -X POST "$QUARK_ACHIEVEMENTS_URL" \
  -H "Authorization: Bearer $QUARK_ACHIEVEMENTS_TOKEN" \
  -d '{"game":"'"$QUARK_GAME_ID"'","achievement":"ACH_FIRST","name":"First step"}'
```

- The server listens only on `127.0.0.1`.
- The token changes on every launch and is masked in the session log.
- The launched game receives `QUARK_ACHIEVEMENTS_URL`,
  `QUARK_ACHIEVEMENTS_TOKEN` and `QUARK_GAME_ID`.
- The game must never log the token.

This API is for games that have no usable achievement source. It does not
replace official Steam achievements.

## Epic overlay (EOS)

In **Settings → Stores & tools → Epic overlay (EOS)**, Quark can download the EOS
overlay once and enable it in the game prefix. This is the mechanism Heroic
uses: the Epic overlay appears in-game with `Shift+F3`.

This feature depends on the game, prefix and EOS version. It is not required for
achievements: they are read through Legendary even without the EOS overlay.

## Steam achievements

Steam provides no readable achievement source without a Steam session. Quark
shows the Steam tab only when a usable source exists. Steam Cloud and Steam
achievements remain managed by the Steam client.

## Troubleshooting

| Problem | Check |
| --- | --- |
| Achievements stay empty | Check the Epic/GOG connection and metadata cache; refresh the game page. |
| A local achievement does not arrive | Check the URL, token and `QUARK_GAME_ID`; the server only listens on loopback. |
| EOS does not appear | Check the active prefix and reinstall the EOS overlay. |

## See also

- [Stores and saves](Stores-and-Saves)
- [Proton, Wine and options](Proton-Wine-and-Launch-Options)
- [Settings](Settings)
- [Troubleshooting](Troubleshooting)

---

[Home](Home) · [Stores](Stores-and-Saves) · [Settings](Settings)
