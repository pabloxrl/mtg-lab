package forge.game;

import com.google.gson.*;
import com.google.common.collect.Table;
import forge.game.card.Card;
import forge.game.combat.Combat;
import forge.game.cost.Cost;
import forge.game.cost.CostPart;
import forge.game.cost.CostDecisionMakerBase;
import forge.game.event.Event;
import forge.game.mana.Mana;
import forge.game.player.Player;
import forge.game.spellability.SpellAbility;
import forge.game.spellability.SpellAbilityStackInstance;
import forge.game.spellability.TargetChoices;
import forge.game.zone.ZoneType;
import java.io.*;
import java.nio.file.*;
import java.util.*;
import java.util.zip.GZIPOutputStream;

/** Original, privileged observational bridge. No AI or rules decisions live here. */
public final class MtgRawCapture {
    private static MtgRawCapture active;
    private static final Set<String> SUPPORTED = Set.of(
        "cheatShuffle", "complainCardsCantPlayWell", "revealUnsupported", "awaitNextInput",
        "chooseStartingPlayer", "mulliganKeepHand", "tuckCardsViaMulligan", "willPutCardOnTop",
        "chooseSpellAbilityToPlay", "playChosenSpellAbility", "resetAtEndOfTurn", "autoPassCancel",
        "payManaCost", "declareAttackers", "declareBlockers", "assignCombatDamage",
        "orderAndPlaySimultaneousSa", "orderSimultaneousSa", "playSpellAbilityNoStack",
        "chooseModeForAbility", "orderMoveToZoneList", "chooseCardsToDiscardToMaximumHandSize",
        "getCostDecisionMaker", "confirmAction", "confirmTrigger", "playTrigger", "chooseTargetsFor",
        "chooseCardsForCost", "chooseOptionalCosts", "orderCosts", "chooseCardsToDiscardFrom",
        "getAbilityToPlay", "orderBlockers", "orderAttackers", "orderBlocker", "cost.visit", "cost.paysRightAfterDecision");
    private final Game game;
    private final BufferedWriter writer;
    private long sequence;
    private long calls;
    private long randomCalls;
    private final long budget;
    private final Map<Card, Integer> instances = new IdentityHashMap<>();
    private boolean emitting;
    private boolean failed;
    private boolean truncated;
    private String firstError;

    private MtgRawCapture(Game game, Path path, long budget) throws IOException {
        this.game = game;
        this.budget = budget;
        writer = path == null ? null : new BufferedWriter(new OutputStreamWriter(new GZIPOutputStream(
                Files.newOutputStream(path, StandardOpenOption.CREATE_NEW)), java.nio.charset.StandardCharsets.UTF_8));
    }
    public static JsonObject audit(Game game) {
        try { return new MtgRawCapture(game, null, Long.MAX_VALUE).state(); }
        catch (IOException e) { throw new java.io.UncheckedIOException(e); }
    }
    public static void open(Game game, Path path, long budget) throws IOException {
        if (active != null) throw new IllegalStateException("capture already open");
        active = new MtgRawCapture(game, path, budget);
        JsonObject header = new JsonObject(); header.addProperty("schema_version", 1);
        active.emit("start", header);
    }
    public static void close(String status) throws IOException {
        if (active == null) return;
        try {
            JsonObject row = new JsonObject(); row.addProperty("status", active.truncated ? "truncated" : active.failed ? "failed" : status);
            row.addProperty("first_error", active.firstError);
            row.addProperty("callback_count", active.calls);
            row.addProperty("random_count", active.randomCalls);
            row.addProperty("record_count", active.sequence + 1);
            active.emit("finish", row);
        } finally {
            try { active.writer.close(); } finally { active = null; }
        }
    }
    public static long enter(Player player, String id, String method, Object[] args) {
        if (active == null || player.getGame() != active.game) return -1;
        if (!SUPPORTED.contains(method) || method.equals(System.getProperty("mtglab.rejectCallback"))) {
            active.failed = true;
            active.firstError = "unsupported raw callback: " + method;
            JsonObject unsupported = new JsonObject(); unsupported.addProperty("method", method);
            active.emit("unsupported", unsupported);
            throw new IllegalArgumentException("unsupported raw callback: " + method);
        }
        if (active.failed) throw new IllegalStateException("raw capture already failed: " + active.firstError);
        long call = ++active.calls;
        if (call > active.budget) {
            active.truncated = true;
            active.firstError = "raw capture budget: truncated";
            active.emit("truncation", new JsonObject());
            throw new IllegalStateException(active.firstError);
        }
        JsonObject row = new JsonObject();
        row.addProperty("call", call); row.addProperty("method_id", id); row.addProperty("method", method);
        row.add("actor", active.value(player)); row.add("arguments", active.value(Arrays.asList(args)));
        active.emit("enter", row);
        return call;
    }
    public static void leave(Player player, long call, Object result, Object[] args) {
        if (call < 0) return;
        JsonObject row = new JsonObject(); row.addProperty("call", call);
        row.add("result", active.value(result)); row.add("arguments", active.value(Arrays.asList(args)));
        active.emit("leave", row);
    }
    public static void failure(Player player, long call, Throwable error) {
        if (active == null || call < 0) return;
        JsonObject row = new JsonObject(); row.addProperty("call", call);
        active.failed = true;
        row.addProperty("error", error.toString()); active.emit("failure", row);
    }
    public static void event(Game game, Event event) {
        if (active == null || active.game != game || active.emitting) return;
        JsonObject row = new JsonObject(); row.addProperty("event", event.getClass().getSimpleName());
        row.add("payload", active.value(event));
        active.emit("event", row);
    }
    public static void boundary(Game game, String boundary) {
        if (active == null || game != active.game) return;
        JsonObject row = new JsonObject(); row.addProperty("boundary", boundary);
        active.emit("boundary", row);
    }
    public static void shuffle(Player player, String stage) {
        if (active == null || player.getGame() != active.game) return;
        JsonObject row = new JsonObject(); row.add("actor", active.value(player));
        row.addProperty("stage", stage); row.add("library", active.value(player.getCardsIn(ZoneType.Library)));
        active.emit("shuffle", row);
    }
    public static void random(int bits, int result) {
        if (active == null) return;
        JsonObject row = new JsonObject(); row.addProperty("bits", bits); row.addProperty("result", result);
        row.addProperty("draw", ++active.randomCalls);
        active.emit("random", row);
    }
    public static final class ObservedRandom extends Random {
        public ObservedRandom(long seed) { super(seed); }
        @Override protected int next(int bits) {
            int result = super.next(bits);
            MtgRawCapture.random(bits, result);
            return result;
        }
    }
    private void emit(String kind, JsonObject row) {
        if (emitting) throw new IllegalStateException("recursive raw observation");
        emitting = true;
        try {
            row.addProperty("index", sequence++); row.addProperty("kind", kind);
            if (!kind.equals("random")) row.add("state", state());
            writer.write(row.toString()); writer.newLine(); writer.flush();
        } catch (IOException e) { throw new java.io.UncheckedIOException(e); }
        finally { emitting = false; }
    }
    private JsonObject state() {
        JsonObject result = new JsonObject();
        result.addProperty("turn", game.getPhaseHandler().getTurn());
        result.addProperty("phase", String.valueOf(game.getPhaseHandler().getPhase()));
        result.add("active", value(game.getPhaseHandler().getPlayerTurn()));
        result.add("priority", value(game.getPhaseHandler().getPriorityPlayer()));
        result.addProperty("terminal", game.isGameOver());
        JsonArray players = new JsonArray();
        for (Player p : game.getRegisteredPlayers()) {
            JsonObject row = new JsonObject(); row.add("player", value(p)); row.addProperty("life", p.getLife());
            JsonObject zones = new JsonObject();
            for (ZoneType zone : ZoneType.values()) {
                if (zone == ZoneType.Stack) continue;
                if (p.getZone(zone) != null) zones.add(zone.name(), value(p.getCardsIn(zone)));
            }
            row.add("zones", zones);
            if (p.getOutcome() != null) {
                JsonObject outcome = new JsonObject(); outcome.addProperty("won", p.getOutcome().hasWon());
                outcome.add("loss", value(p.getOutcome().lossState)); row.add("outcome", outcome);
            } else { row.add("outcome", JsonNull.INSTANCE); }
            JsonArray mana = new JsonArray();
            for (Mana m : p.getManaPool()) mana.add(value(m));
            row.add("mana", mana); players.add(row);
        }
        result.add("players", players);
        JsonArray stack = new JsonArray();
        for (SpellAbilityStackInstance item : game.getStack()) stack.add(value(item.getSpellAbility()));
        result.add("stack", stack);
        result.add("combat", value(game.getCombat()));
        try {
            java.lang.reflect.Field field = forge.game.trigger.TriggerHandler.class.getDeclaredField("waitingTriggers");
            field.setAccessible(true);
            result.add("pending_triggers", value(field.get(game.getTriggerHandler())));
        } catch (ReflectiveOperationException e) { throw new IllegalStateException("unobservable trigger queue", e); }
        return result;
    }
    private JsonObject card(Card card) {
        JsonObject row = new JsonObject();
        row.addProperty("id", card.getId());
        row.addProperty("instance", instances.computeIfAbsent(card, c -> instances.size()));
        row.addProperty("timestamp", card.getGameTimestamp());
        row.addProperty("name", card.getName());
        row.add("printing", value(card.getPaperCard()));
        row.add("owner", value(card.getOwner())); row.add("controller", value(card.getController()));
        row.addProperty("zone", card.getZone() == null ? null : card.getZone().getZoneType().name());
        row.addProperty("token", card.isToken()); row.addProperty("tapped", card.isTapped());
        row.addProperty("sick", card.isSick()); row.addProperty("damage", card.getDamage());
        row.addProperty("power", card.getNetPower()); row.addProperty("toughness", card.getNetToughness());
        row.addProperty("types", card.getType().toString()); row.addProperty("colors", card.getColor().getColor());
        JsonArray keywords = new JsonArray();
        for (forge.game.keyword.KeywordInterface kw : card.getKeywords()) keywords.add(kw.getOriginal());
        row.add("keywords", keywords);
        row.add("pt_boosts", value(card.getPTBoostTable()));
        row.add("keyword_changes", value(card.getChangedCardKeywords()));
        return row;
    }
    private Object field(Object object, Class<?> type, String name) {
        try {
            java.lang.reflect.Field field = type.getDeclaredField(name); field.setAccessible(true);
            return field.get(object);
        } catch (ReflectiveOperationException e) { throw new IllegalStateException("unobservable source field: " + name, e); }
    }
    private JsonElement value(Object v) {
        if (v == null) return JsonNull.INSTANCE;
        if (v instanceof Boolean) return new JsonPrimitive((Boolean)v);
        if (v instanceof Number) return new JsonPrimitive((Number)v);
        if (v instanceof String || v instanceof Enum<?>) return new JsonPrimitive(v.toString());
        if (v instanceof Player p) { JsonObject r = new JsonObject(); r.addProperty("player_id", p.getId()); return r; }
        if (v instanceof Card c) return card(c);
        if (v instanceof SpellAbility sa) {
            JsonObject r = new JsonObject(); r.addProperty("id", sa.getId());
            r.add("host", card(sa.getHostCard())); r.add("actor", value(sa.getActivatingPlayer()));
            r.addProperty("api", String.valueOf(sa.getApi())); r.addProperty("spell", sa.isSpell());
            r.addProperty("mana_ability", sa.isManaAbility()); r.addProperty("trigger", sa.isTrigger());
            r.addProperty("cost", String.valueOf(sa.getPayCosts()));
            r.add("targets", value(sa.getTargets())); r.add("paid", value(sa.getPaidHash()));
            r.add("paying_mana", value(sa.getPayingMana()));
            r.add("paying_mana_abilities", value(sa.getPayingManaAbilities()));
            r.add("optional_costs", value(sa.getOptionalCosts()));
            r.add("trigger_context", value(sa.getTriggeringObjects()));
            r.add("sub", value(sa.getSubAbility())); return r;
        }
        if (v instanceof TargetChoices t) {
            JsonArray r = new JsonArray();
            for (GameObject target : t) {
                JsonObject item = new JsonObject(); item.add("target", value(target));
                item.add("allocation", value(t.getDividedValue(target))); r.add(item);
            }
            return r;
        }
        if (v instanceof forge.game.trigger.TriggerWaiting waiting) {
            JsonObject r = new JsonObject(); r.add("mode", value(waiting.getMode()));
            r.add("params", value(waiting.getParams()));
            JsonArray triggers = new JsonArray();
            if (waiting.getTriggers() != null) for (forge.game.trigger.Trigger t : waiting.getTriggers()) {
                JsonObject entry = new JsonObject(); entry.add("trigger", value(t));
                entry.add("controller", value(waiting.getController(t))); triggers.add(entry);
            }
            r.add("triggers", triggers); return r;
        }
        if (v instanceof forge.game.trigger.Trigger trigger) {
            JsonObject r = new JsonObject(); r.addProperty("id", trigger.getId());
            r.add("host", value(trigger.getHostCard())); r.add("mode", value(trigger.getMode()));
            r.add("remembered", value(trigger.getTriggerRemembered())); return r;
        }
        if (v instanceof forge.game.keyword.KeywordsChange k) {
            JsonObject r = new JsonObject(); JsonArray added = new JsonArray();
            for (forge.game.keyword.KeywordInterface kw : k.getKeywords()) added.add(kw.getOriginal());
            r.add("added", added); r.add("removed", value(k.getRemoveKeywords()));
            r.addProperty("remove_all", k.isRemoveAllKeywords()); return r;
        }
        if (v instanceof Mana m) {
            JsonObject r = new JsonObject(); r.addProperty("color", m.getColor());
            r.add("source", value(m.getSourceCard())); r.addProperty("restricted", m.isRestricted());
            r.addProperty("source_ability", m.getManaAbility().getSourceSA().getId());
            r.addProperty("restrictions", m.getManaAbility().getManaRestrictions());
            r.addProperty("extra_restrictions", m.getManaAbility().getExtraManaRestriction()); return r;
        }
        if (v instanceof Combat c) {
            JsonArray r = new JsonArray();
            for (Card a : c.getAttackers()) {
                JsonObject pair = new JsonObject(); pair.add("attacker", value(a));
                pair.add("defender", value(c.getDefenderByAttacker(a))); pair.add("blockers", value(c.getBlockers(a))); r.add(pair);
            }
            return r;
        }
        if (v instanceof Table<?,?,?> table) {
            JsonArray r = new JsonArray();
            for (Table.Cell<?,?,?> c : table.cellSet()) r.add(value(Arrays.asList(c.getRowKey(), c.getColumnKey(), c.getValue())));
            return r;
        }
        if (v instanceof com.google.common.collect.Multimap<?,?> map) return value(map.asMap());
        if (v instanceof forge.card.ColorSet colors) return value(colors.getColor());
        if (v instanceof Map.Entry<?,?> entry) return value(Arrays.asList(entry.getKey(), entry.getValue()));
        if (v instanceof forge.item.IPaperCard paper) {
            JsonObject r = new JsonObject(); r.addProperty("name", paper.getName());
            r.addProperty("edition", paper.getEdition()); r.addProperty("collector_number", paper.getCollectorNumber());
            r.addProperty("art_index", paper.getArtIndex()); return r;
        }
        if (v instanceof Map<?,?> map) {
            JsonArray r = new JsonArray();
            for (Map.Entry<?,?> entry : map.entrySet()) r.add(value(Arrays.asList(entry.getKey(), entry.getValue())));
            return r;
        }
        if (v instanceof Iterable<?> list) { JsonArray r = new JsonArray(); for (Object o : list) r.add(value(o)); return r; }
        if (v instanceof org.apache.commons.lang3.tuple.Pair<?,?> pair) return value(Arrays.asList(pair.getLeft(), pair.getRight()));
        if (v instanceof Cost || v instanceof CostPart || v instanceof forge.card.mana.ManaCost) {
            JsonObject r = new JsonObject(); r.addProperty("class", v.getClass().getName());
            r.addProperty("value", v.toString()); return r;
        }
        if (v instanceof forge.game.cost.CostPayment payment) {
            JsonObject r = new JsonObject(); r.add("cost", value(payment.getCost()));
            r.addProperty("ability_id", payment.getAbility().getId());
            r.add("host", value(payment.getAbility().getHostCard()));
            r.add("adjusted", value(field(payment, forge.game.cost.CostPayment.class, "adjustedCost")));
            r.add("paid_parts", value(field(payment, forge.game.cost.CostPayment.class, "paidCostParts")));
            r.add("conversion", value(field(payment, forge.game.mana.ManaConversionMatrix.class, "colorConversionMatrix")));
            r.add("restriction", value(field(payment, forge.game.mana.ManaConversionMatrix.class, "colorRestrictionMatrix")));
            return r;
        }
        if (v instanceof forge.game.mana.ManaConversionMatrix matrix) {
            JsonObject r = new JsonObject();
            r.add("conversion", value(field(matrix, forge.game.mana.ManaConversionMatrix.class, "colorConversionMatrix")));
            r.add("restriction", value(field(matrix, forge.game.mana.ManaConversionMatrix.class, "colorRestrictionMatrix")));
            r.addProperty("snow_for_color", matrix.isSnowForColor()); return r;
        }
        if (v.getClass().isArray()) {
            JsonArray r = new JsonArray();
            for (int i = 0; i < java.lang.reflect.Array.getLength(v); i++) r.add(value(java.lang.reflect.Array.get(v, i)));
            return r;
        }
        if (v instanceof forge.game.cost.PaymentDecision p) {
            JsonObject r = new JsonObject(); r.addProperty("amount", p.c); r.add("type", value(p.type));
            r.add("cards", value(p.cards)); r.add("mana", value(p.mana)); r.add("players", value(p.players));
            r.add("abilities", value(p.sp)); r.add("colors", value(p.colors));
            r.add("counter_table", value(p.counterTable)); r.add("matrix", value(p.matrix)); return r;
        }
        if (v instanceof forge.trackable.TrackableObject t) {
            JsonObject r = new JsonObject(); r.addProperty("view_class", v.getClass().getName());
            r.addProperty("id", t.getId()); return r;
        }
        if (v instanceof forge.game.player.PlayerStatistics p) return value(p.getOutcome());
        if (v instanceof forge.game.player.PlayerOutcome o) {
            JsonObject r = new JsonObject(); r.addProperty("won", o.hasWon());
            r.add("loss", value(o.lossState)); r.add("alternate_win", value(o.altWinSourceName));
            r.add("loss_spell", value(o.loseConditionSpell)); return r;
        }
        if (v instanceof forge.game.player.RegisteredPlayer p) return value(p.getPlayer().getName());
        if (v.getClass().isRecord() && v.getClass().getName().startsWith("forge.game.")) {
            JsonObject r = new JsonObject();
            for (java.lang.reflect.RecordComponent c : v.getClass().getRecordComponents()) {
                try {
                    Object field = c.getAccessor().invoke(v);
                    if (field instanceof String && Set.of("message", "stackDescription").contains(c.getName())) {
                        JsonObject label = new JsonObject();
                        try {
                            label.addProperty("text_sha256", java.util.HexFormat.of().formatHex(
                                java.security.MessageDigest.getInstance("SHA-256").digest(((String)field).getBytes(java.nio.charset.StandardCharsets.UTF_8))));
                        } catch (java.security.NoSuchAlgorithmException e) { throw new IllegalStateException(e); }
                        r.add(c.getName(), label);
                    } else { r.add(c.getName(), value(field)); }
                }
                catch (ReflectiveOperationException e) { throw new IllegalArgumentException("unobservable event", e); }
            }
            return r;
        }
        if (v instanceof CostDecisionMakerBase) return new JsonPrimitive(v.getClass().getName());
        failed = true;
        if (firstError == null) firstError = "unsupported raw value: " + v.getClass().getName();
        throw new IllegalArgumentException(firstError);
    }
}
