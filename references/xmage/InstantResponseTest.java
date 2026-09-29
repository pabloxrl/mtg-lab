package org.mage.test.mtglab;

import com.google.gson.*;
import mage.Mana;
import mage.abilities.Ability;
import mage.abilities.mana.ActivatedManaAbilityImpl;
import mage.abilities.costs.mana.ManaCost;
import mage.cards.Card;
import mage.constants.*;
import mage.game.Game;
import mage.game.combat.CombatGroup;
import mage.util.MultiAmountMessage;
import mage.game.events.GameEvent;
import mage.game.permanent.Permanent;
import mage.game.stack.StackObject;
import mage.game.stack.Spell;
import mage.players.Player;
import mage.target.Target;
import mage.watchers.Watcher;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.junit.runners.Parameterized;
import org.mage.test.player.TestComputerPlayer;
import org.mage.test.player.TestPlayer;
import org.mage.test.serverside.base.CardTestPlayerBase;
import java.nio.file.*;
import java.io.Serializable;
import mage.target.common.TargetDiscard;
import java.nio.charset.StandardCharsets;
import java.util.*;
import static org.junit.Assert.*;

/** Original bounded translator, not an upstream scenario adaptation. */
@RunWith(Parameterized.class)
public class InstantResponseTest extends CardTestPlayerBase {
    private final JsonObject fixture;
    private final Map<String,UUID> ids = new LinkedHashMap<>();
    private final JsonArray checkpoints = new JsonArray(), consumed = new JsonArray();
    private int cursor;
    private final Map<Integer,List<Card>> initialHands = new HashMap<>();
    private boolean observing;
    private final Map<UUID,Integer> initialCounters = new HashMap<>();
    private final Map<UUID,Map<UUID,Integer>> targetIncarnations = new HashMap<>();
    private Spell pendingResolution;
    private JsonElement lastResolution = JsonNull.INSTANCE;
    private int incarnation(UUID id, Game game) {
        assertTrue("missing initial identity", initialCounters.containsKey(id));
        return game.getCard(id).getZoneChangeCounter(game)-initialCounters.get(id);
    }
    private void observeResolution(Game game) {
        if(pendingResolution==null)return;
        for(StackObject so:game.getStack())if(so.getId().equals(pendingResolution.getId()))return;
        assertTrue("removed spell never attempted resolution",pendingResolution.isResolving());
        int legal=0;
        for(Target t:pendingResolution.getSpellAbility().getTargets())legal+=t.getTargets().size();
        JsonObject r=new JsonObject();r.addProperty("id",semantic(pendingResolution.getSourceId()));
        r.addProperty("legal_targets",legal);r.addProperty("resolved",!pendingResolution.isCountered());
        lastResolution=r;pendingResolution=null;
    }
    @Parameterized.Parameters(name="{index}") public static Collection<Object[]> cases() throws Exception {
        JsonObject d=JsonParser.parseString(new String(Files.readAllBytes(Paths.get(System.getProperty("mtglab.fixture"))),StandardCharsets.UTF_8)).getAsJsonObject();
        keys(d,"version","cases"); assertEquals(1,d.get("version").getAsInt());
        List<Object[]> result=new ArrayList<>();
        for(JsonElement e:d.getAsJsonArray("cases")) result.add(new Object[]{e.getAsJsonObject()});
        return result;
    }
    public InstantResponseTest(JsonObject fixture) { this.fixture=fixture; }
    private static void keys(JsonObject o,String... names) { assertEquals(new HashSet<>(Arrays.asList(names)),o.keySet()); }
    private JsonArray script() { return fixture.getAsJsonArray("script"); }
    private String kind() { assertTrue("missing choice at "+cursor,cursor<script().size()); return script().get(cursor).getAsJsonObject().get("kind").getAsString(); }
    private JsonObject take(String kind,int seat,String... fields) {
        assertEquals("choice order at "+cursor,kind,kind()); JsonObject a=script().get(cursor).getAsJsonObject();
        List<String> ks=new ArrayList<>(Arrays.asList(fields));ks.add("kind");ks.add("actor");keys(a,ks.toArray(new String[0]));
        assertEquals("wrong actor at "+cursor,seat,a.get("actor").getAsInt()); cursor++;consumed.add(a.deepCopy());return a;
    }
    private UUID player(int seat) { assertTrue(seat==0||seat==1);return seat==0?playerA.getId():playerB.getId(); }
    private int seat(UUID id) { if(id.equals(playerA.getId()))return 0;if(id.equals(playerB.getId()))return 1;throw new AssertionError("unknown seat"); }
    private UUID object(String id) { assertTrue("unknown object "+id,ids.containsKey(id));return ids.get(id); }
    private String semantic(UUID id) { return ids.entrySet().stream().filter(e->e.getValue().equals(id)).map(Map.Entry::getKey).findFirst().orElseThrow(()->new AssertionError("unknown object "+id)); }
    public static class DamageObserver extends Watcher {
        private JsonArray events = new JsonArray();
        public DamageObserver(){super(WatcherScope.GAME);}
        private DamageObserver(DamageObserver other){super(other);events=other.events.deepCopy();}
        @Override public Watcher copy(){return new DamageObserver(this);}
        @Override public void watch(GameEvent event,Game game){
            if(event.getType()==GameEvent.EventType.DAMAGED_PERMANENT){
                JsonObject d=new JsonObject();d.addProperty("source",event.getSourceId().toString());d.addProperty("target",event.getTargetId().toString());d.addProperty("amount",event.getAmount());events.add(d);
            }
        }
    }
    @Override protected TestPlayer createPlayer(String name,RangeOfInfluence range) {
        return new TestPlayer(new TestComputerPlayer(name,range)) {
            @Override public boolean priority(Game game) {
                assertTrue(game.getTurnNum()==1||game.getTurnNum()==2);
                assertTrue("unsupported step",Arrays.asList(PhaseStep.UPKEEP,PhaseStep.PRECOMBAT_MAIN,
                    PhaseStep.BEGIN_COMBAT,PhaseStep.DECLARE_ATTACKERS,PhaseStep.DECLARE_BLOCKERS,
                    PhaseStep.COMBAT_DAMAGE,PhaseStep.END_COMBAT,PhaseStep.POSTCOMBAT_MAIN,PhaseStep.END_TURN).contains(game.getTurnStepType()));
                int s=seat(getId());
                if(!observing){
                    // Install synthetic hands at the declared boundary, not before a
                    // hidden mulligan phase. No opening-game execution is claimed.
                    for(int owner=0;owner<2;owner++) game.cheat(player(owner),Collections.emptyList(),initialHands.get(owner),Collections.emptyList(),Collections.emptyList(),Collections.emptyList(),Collections.emptyList());
                    game.getState().addWatcher(new DamageObserver());
                    for(UUID id:ids.values())initialCounters.put(id,game.getCard(id).getZoneChangeCounter(game));
                    observing=true;
                }
                observeResolution(game);
                while(cursor<script().size()&&kind().equals("checkpoint")){
                    JsonObject c=script().get(cursor).getAsJsonObject();keys(c,"kind","name");
                    checkpoints.add(checkpoint(c.get("name").getAsString(),game));cursor++;consumed.add(c.deepCopy());
                }
                if(cursor==script().size()){assertTrue("unfinished stack",game.getStack().isEmpty());game.pause();return false;}
                switch(kind()){
                    case "pass":
                        take("pass",s);
                        if(!game.getStack().isEmpty())pendingResolution=(Spell)game.getStack().peek();
                        pass(game);return true;
                    case "mana":{
                        JsonObject a=take("mana",s,"source");Permanent p=game.getPermanent(object(a.get("source").getAsString()));assertNotNull(p);assertEquals(getId(),p.getControllerId());
                        List<ActivatedManaAbilityImpl> abilities=p.getAbilities().getActivatedManaAbilities(Zone.BATTLEFIELD);assertEquals(1,abilities.size());
                        assertTrue("illegal mana activation",activateAbility(abilities.get(0),game));return true;
                    }
                    case "cast":{
                        JsonObject a=take("cast",s,"source");Card card=game.getCard(object(a.get("source").getAsString()));assertTrue(game.getPlayer(getId()).getHand().contains(card.getId()));
                        getManaPool().setAutoPayment(false);
                        assertTrue("illegal cast",cast(card.getSpellAbility(),game,false,null));
                        take("finish_cast",s);
                        Spell spell=(Spell)game.getStack().peek();
                        assertEquals(card.getId(),spell.getSourceId());
                        Map<UUID,Integer> incarnations=new HashMap<>();
                        for(Target t:spell.getSpellAbility().getTargets())for(UUID id:t.getTargets())incarnations.put(id,incarnation(id,game));
                        targetIncarnations.put(spell.getSourceId(),incarnations);
                        return true;
                    }
                    default:throw new AssertionError("unexpected priority choice "+kind());
                }
            }
            @Override public void selectAttackers(Game game,UUID attackingPlayerId) {
                assertEquals(getId(),attackingPlayerId);
                JsonObject a=take("attackers",seat(getId()),"attackers");
                Set<UUID> selected=new HashSet<>();
                for(JsonElement e:a.getAsJsonArray("attackers")) {
                    UUID id=object(e.getAsString());Permanent p=game.getPermanent(id);
                    assertTrue("duplicate attacker",selected.add(id));
                    assertNotNull("illegal attacker",p);
                    assertEquals("illegal attacker controller",getId(),p.getControllerId());
                    assertTrue("illegal attacker",game.getPlayer(getId()).getAvailableAttackers(game).contains(p)
                        &&p.canAttack(player(1-seat(getId())),game));
                    declareAttacker(id,player(1-seat(getId())),game,false);
                    assertNotNull("attacker not declared",game.getCombat().findGroup(id));
                }
            }
            @Override public void selectBlockers(Ability source,Game game,UUID defendingPlayerId) {
                assertEquals(getId(),defendingPlayerId);
                JsonObject a=take("blockers",seat(getId()),"blocks");
                Set<UUID> selected=new HashSet<>();
                for(JsonElement e:a.getAsJsonArray("blocks")) {
                    JsonObject b=e.getAsJsonObject();keys(b,"blocker","attacker");
                    UUID blocker=object(b.get("blocker").getAsString()),attacker=object(b.get("attacker").getAsString());
                    Permanent p=game.getPermanent(blocker);CombatGroup group=game.getCombat().findGroup(attacker);
                    assertTrue("duplicate blocker",selected.add(blocker));
                    assertNotNull("illegal blocker",p);assertNotNull("illegal block target",group);
                    assertEquals("illegal blocker controller",getId(),p.getControllerId());
                    assertTrue("illegal blocker",p.canBlock(attacker,game)&&group.canBlock(p,game));
                    declareBlocker(getId(),blocker,attacker,game);
                    assertTrue("blocker not declared",group.getBlockers().contains(blocker));
                }
            }
            @Override public List<Integer> getMultiAmountWithIndividualConstraints(Outcome outcome,
                    List<MultiAmountMessage> messages,int totalMin,int totalMax,MultiAmountType type,Game game) {
                throw new AssertionError("unscripted combat damage allocation");
            }
            @Override public boolean choose(Outcome outcome,Target target,Ability source,Game game,Map<String,Serializable> options){
                assertTrue("unexpected non-target choice",target instanceof TargetDiscard);
                assertEquals(PhaseStep.CLEANUP,game.getTurnStepType());
                assertEquals(getId(),game.getActivePlayerId());
                assertEquals(Outcome.Discard,outcome);
                while(cursor<script().size()&&kind().equals("checkpoint")){
                    JsonObject c=script().get(cursor).getAsJsonObject();keys(c,"kind","name");
                    JsonObject point=checkpoint(c.get("name").getAsString(),game);
                    JsonObject state=point.getAsJsonObject("state");state.add("priority",JsonNull.INSTANCE);
                    JsonObject discard=new JsonObject();discard.addProperty("actor",seat(getId()));discard.addProperty("count",target.getMinNumberOfTargets());state.add("discard",discard);
                    checkpoints.add(point);cursor++;consumed.add(c.deepCopy());
                }
                JsonObject a=take("discard",seat(getId()),"cards");
                JsonArray cards=a.getAsJsonArray("cards");
                assertEquals("discard count",target.getMinNumberOfTargets(),cards.size());
                assertEquals("discard count",target.getMaxNumberOfTargets(),cards.size());
                Set<UUID> selected=new HashSet<>();
                for(JsonElement card:cards){
                    UUID id=object(card.getAsString());
                    assertTrue("illegal discard card",getHand().contains(id));
                    assertTrue("duplicate discard card",selected.add(id));
                    target.addTarget(id,source,game);
                    assertTrue("illegal discard selection",target.getTargets().contains(id));
                }
                return true;
            }
            @Override public boolean chooseTarget(Outcome outcome,Target target,Ability source,Game game){
                JsonObject a=take("target",seat(getId()),"target");UUID id=object(a.get("target").getAsString());
                assertTrue("illegal target",target.canTarget(id,source,game));target.addTarget(id,source,game);return true;
            }
            @Override public boolean playMana(Ability ability,ManaCost unpaid,String prompt,Game game){
                JsonObject a=take("pay",seat(getId()),"color");String color=a.get("color").getAsString();
                assertFalse("automatic payment forbidden",getManaPool().isAutoPayment());
                if(color.equals("G")){assertTrue(getManaPool().getGreen()>0);getManaPool().unlockManaType(ManaType.GREEN);}
                else if(color.equals("R")){assertTrue(getManaPool().getRed()>0);getManaPool().unlockManaType(ManaType.RED);}
                else throw new AssertionError("unsupported payment");
                return true;
            }
            @Override public boolean chooseMulligan(Game game){throw new AssertionError("unexpected mulligan");}
        };
    }
    private JsonObject checkpoint(String name,Game game){
        JsonObject state=new JsonObject();state.addProperty("turn",game.getTurnNum());state.addProperty("step",game.getTurnStepType().name().toLowerCase(Locale.ROOT));state.addProperty("active",seat(game.getActivePlayerId()));state.addProperty("priority",seat(game.getPriorityPlayerId()));
        assertTrue("unexpected exile",game.getExile().getAllCards(game).isEmpty());
        Set<UUID> observed=new HashSet<>();
        for(Permanent p:game.getBattlefield().getAllActivePermanents())observed.add(p.getId());
        for(int owner=0;owner<2;owner++){Player p=game.getPlayer(player(owner));observed.addAll(p.getHand());observed.addAll(p.getGraveyard());}
        for(StackObject spell:game.getStack())observed.add(spell.getSourceId());
        assertEquals("missing/extra object",new HashSet<>(ids.values()),observed);
        JsonArray life=new JsonArray(),mana=new JsonArray(),objects=new JsonArray(),stack=new JsonArray();
        for(int s=0;s<2;s++){
            Player p=game.getPlayer(player(s));life.add(p.getLife());Mana m=p.getManaPool().getMana();JsonArray pool=new JsonArray();
            for(int n:new int[]{m.getWhite(),m.getBlue(),m.getBlack(),m.getRed(),m.getGreen(),m.getColorless()})pool.add(n);mana.add(pool);
            assertTrue(p.getLibrary().getCardList().isEmpty());
        }
        for(JsonElement e:fixture.getAsJsonObject("setup").getAsJsonArray("objects")){
            JsonObject spec=e.getAsJsonObject(),o=spec.deepCopy();UUID id=object(spec.get("id").getAsString());Permanent p=game.getPermanent(id);
            assertEquals(player(spec.get("owner").getAsInt()),game.getCard(id).getOwnerId());
            if(p!=null)assertEquals(player(spec.get("owner").getAsInt()),p.getControllerId());
            Zone z=game.getState().getZone(id);String zone;
            switch(z){case BATTLEFIELD:zone="battlefield";break;case HAND:zone="hand";break;case STACK:zone="stack";break;case GRAVEYARD:zone="graveyard";break;default:throw new AssertionError("unsupported zone "+z);}
            o.addProperty("incarnation",incarnation(id,game));
            o.addProperty("zone",zone);o.addProperty("tapped",p!=null&&p.isTapped());
            if(p!=null&&p.isCreature(game)){o.addProperty("power",p.getPower().getValue());o.addProperty("toughness",p.getToughness().getValue());o.addProperty("damage",p.getDamage());}
            else{ o.add("power",JsonNull.INSTANCE);o.add("toughness",JsonNull.INSTANCE);o.add("damage",JsonNull.INSTANCE); }
            objects.add(o);
        }
        List<StackObject> spells=new ArrayList<>();for(StackObject s:game.getStack())spells.add(s);Collections.reverse(spells);
        for(StackObject so:spells){assertTrue(so instanceof Spell);Spell spell=(Spell)so;JsonObject s=new JsonObject();s.addProperty("id",semantic(spell.getSourceId()));s.addProperty("controller",seat(spell.getControllerId()));JsonArray ts=new JsonArray();
            for(Target t:spell.getSpellAbility().getTargets())for(UUID id:t.getTargets()){
                JsonObject target=new JsonObject();target.addProperty("id",semantic(id));
                Integer incarnation=targetIncarnations.get(spell.getSourceId()).get(id);assertNotNull("missing target identity",incarnation);
                target.addProperty("incarnation",incarnation);ts.add(target);
            }s.add("targets",ts);stack.add(s);
        }
        state.add("life",life);state.add("mana",mana);state.add("objects",objects);state.add("stack",stack);JsonArray observedDamage=new JsonArray();
        for(JsonElement event:game.getState().getWatcher(DamageObserver.class).events){JsonObject d=event.getAsJsonObject().deepCopy();d.addProperty("source",semantic(UUID.fromString(d.get("source").getAsString())));d.addProperty("target",semantic(UUID.fromString(d.get("target").getAsString())));observedDamage.add(d);}
        JsonArray combat=new JsonArray();
        for(CombatGroup group:game.getCombat().getGroups()) {
            for(UUID attacker:group.getAttackers()) {
                JsonObject attack=new JsonObject(),identity=new JsonObject();
                identity.addProperty("id",semantic(attacker));identity.addProperty("incarnation",incarnation(attacker,game));
                attack.add("attacker",identity);attack.addProperty("blocked",group.getBlocked());
                JsonArray blockers=new JsonArray();
                for(UUID blocker:group.getBlockers()) {
                    JsonObject b=new JsonObject();b.addProperty("id",semantic(blocker));
                    b.addProperty("incarnation",incarnation(blocker,game));blockers.add(b);
                }
                attack.add("blockers",blockers);combat.add(attack);
            }
        }
        state.add("combat",combat);
        JsonObject zones=new JsonObject();JsonArray hands=new JsonArray(),graves=new JsonArray();
        for(int owner=0;owner<2;owner++){
            JsonArray hand=new JsonArray(),grave=new JsonArray();Player p=game.getPlayer(player(owner));
            for(UUID id:p.getHand())hand.add(semantic(id));
            for(UUID id:p.getGraveyard())grave.add(semantic(id));
            hands.add(hand);graves.add(grave);
        }
        zones.add("hand",hands);zones.add("graveyard",graves);state.add("ordered_zones",zones);
        state.add("damage_events",observedDamage);state.add("last_resolution",lastResolution.deepCopy());
        JsonObject out=new JsonObject();out.addProperty("name",name);out.add("state",state);return out;
    }
    @Test public void instant() throws Exception {
        keys(fixture,"id","setup","script");JsonObject setup=fixture.getAsJsonObject("setup");keys(setup,"boundary","active","priority","life","mana","libraries","objects");
        assertEquals("turn-1-upkeep-priority",setup.get("boundary").getAsString());assertEquals(0,setup.get("active").getAsInt());assertEquals(0,setup.get("priority").getAsInt());
        assertEquals(JsonParser.parseString("[[0,0,0,0,0,0],[0,0,0,0,0,0]]"),setup.get("mana"));assertEquals(JsonParser.parseString("[[],[]]"),setup.get("libraries"));
        setStrictChooseMode(true);currentGame.setStartingPlayerId(playerA.getId());gameOptions.skipInitShuffling=true;
        for(int s=0;s<2;s++){TestPlayer p=s==0?playerA:playerB;removeAllCardsFromHand(p);removeAllCardsFromLibrary(p);setLife(p,setup.getAsJsonArray("life").get(s).getAsInt());}
        Map<String,String> names=new HashMap<>();names.put("bear-cub","Bear Cub");names.put("forest","Forest");names.put("mountain","Mountain");names.put("bite-down","Bite Down");names.put("giant-growth","Giant Growth");
        for(JsonElement e:setup.getAsJsonArray("objects")){
            JsonObject o=e.getAsJsonObject();keys(o,"id","card","owner","zone");String key=o.get("card").getAsString();assertTrue(names.containsKey(key));int s=o.get("owner").getAsInt();TestPlayer p=s==0?playerA:playerB;player(s);
            String z=o.get("zone").getAsString();assertTrue(z.equals("hand")||z.equals("battlefield"));
            Zone zone=z.equals("hand")?Zone.HAND:Zone.BATTLEFIELD;addCard(zone,p,"FDN-"+names.get(key),1);
            Card card=zone==Zone.HAND?getHandCards(p).get(getHandCards(p).size()-1):getBattlefieldCards(p).get(getBattlefieldCards(p).size()-1).getCard();
            assertNull(ids.put(o.get("id").getAsString(),card.getId()));
        }
        for(int owner=0;owner<2;owner++){TestPlayer p=owner==0?playerA:playerB;initialHands.put(owner,new ArrayList<>(getHandCards(p)));getHandCards(p).clear();}
        setStopAt(2,PhaseStep.UPKEEP);execute();assertEquals(script().size(),cursor);
        JsonObject result=new JsonObject();result.add("checkpoints",checkpoints);result.add("consumed",consumed);
        Files.write(Paths.get(System.getProperty("mtglab.output"),fixture.get("id").getAsString()+".json"),new GsonBuilder().serializeNulls().setPrettyPrinting().create().toJson(result).getBytes(StandardCharsets.UTF_8));
    }
}
