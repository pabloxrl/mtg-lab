package org.mage.test.mtglab;

import com.google.gson.*;
import java.io.*;
import java.nio.charset.StandardCharsets;
import java.nio.file.*;
import java.util.*;
import mage.MageObject;
import mage.Mana;
import mage.abilities.costs.Cost;
import mage.players.ManaPool;
import mage.players.ManaPoolItem;
import mage.game.events.ZoneChangeEvent;
import mage.abilities.*;
import mage.abilities.costs.mana.ManaCost;
import mage.cards.*;
import mage.cards.decks.Deck;
import mage.choices.Choice;
import mage.constants.*;
import mage.game.*;
import mage.game.combat.CombatGroup;
import mage.game.events.GameEvent;
import mage.game.mulligan.LondonMulligan;
import mage.game.permanent.Permanent;
import mage.game.stack.*;
import mage.player.ai.ComputerPlayer7;
import mage.players.Player;
import mage.target.*;
import mage.ApprovingObject;
import mage.util.MultiAmountMessage;
import mage.util.RandomUtil;
import org.junit.Test;

/** Privileged raw source capture, never a normalized replay or a rules oracle.
 * All gameplay choices delegate to the pinned ComputerPlayer7 implementation.
 * Object identity, not a copied game UUID, gates observation of live play.
 */
public class RawAiCaptureTest {
    static final Gson JSON = new GsonBuilder().serializeNulls().create();
    static Recorder recorder;
    static Game firstPriority;
    static UUID firstActor;
    static class Stop extends RuntimeException { Stop(String s){super(s);} }
    static JsonObject object(Object... pairs) {
        JsonObject o=new JsonObject();
        for(int i=0;i<pairs.length;i+=2)o.add((String)pairs[i],describe(pairs[i+1]));
        return o;
    }
    static JsonElement describe(Object v) {
        if(v==null)return JsonNull.INSTANCE;
        if(v instanceof JsonElement)return ((JsonElement)v).deepCopy();
        if(v instanceof String||v instanceof UUID||v instanceof Enum)return new JsonPrimitive(v.toString());
        if(v instanceof Number)return new JsonPrimitive((Number)v);
        if(v instanceof Boolean)return new JsonPrimitive((Boolean)v);
        if(v instanceof Ability){
            Ability a=(Ability)v;
            return object("class",a.getClass().getName(),"id",a.getId(),"source",a.getSourceId(),
                "controller",a.getControllerId(),"source_zcc",a.getStackMomentSourceZCC(),
                "type",a.getAbilityType(),"targets",a.getAllSelectedTargets(),"modes",a.getModes().getSelectedModes(),
                "mana_costs",new ArrayList<>(a.getManaCosts()),"mana_to_pay",new ArrayList<>(a.getManaCostsToPay()),
                "costs",new ArrayList<>(a.getCosts()));
        }
        if(v instanceof Target){Target t=(Target)v;JsonArray selected=new JsonArray();for(UUID id:t.getTargets())selected.add(object("id",id,"amount",t.getTargetAmount(id)));return object("class",t.getClass().getName(),"targets",selected,"min",t.getMinNumberOfTargets(),"max",t.getMaxNumberOfTargets(),"not_target",t.isNotTarget(),"zone_change_counters",targetCounters(t));}
        if(v instanceof Choice)throw new Stop("unsupported callback: textual Choice "+v.getClass().getName());
        if(v instanceof Mode){Mode m=(Mode)v;return object("id",m.getId(),"targets",m.getTargets(),"effects",nativeFields(m.getEffects(),new IdentityHashMap<>()));}
        if(v instanceof MultiAmountType){MultiAmountType t=(MultiAmountType)v;return object("title",t.getTitle(),"header",t.getHeader(),"can_cancel",t.isCanCancel());}
        if(v instanceof MultiAmountMessage){MultiAmountMessage m=(MultiAmountMessage)v;return object("message",textLabel(m.message),"min",m.min,"max",m.max,"default",m.defaultValue);}
        if(v instanceof Mana){Mana m=(Mana)v;return object("white",m.getWhite(),"blue",m.getBlue(),"black",m.getBlack(),"red",m.getRed(),"green",m.getGreen(),"colorless",m.getColorless(),"generic",m.getGeneric(),"any",m.getAny(),"flag",m.getFlag());}
        if(v instanceof ManaPoolItem){ManaPoolItem m=(ManaPoolItem)v;if(m.isConditional())throw new Stop("unsupported conditional mana");return object("source",m.getSourceId(),"original_ability",m.getOriginalId(),"mana",m.getMana(),"duration",m.getDuration(),"stock",m.getStock(),"flag",m.getFlag());}
        if(v instanceof ManaPool){ManaPool m=(ManaPool)v;return object("mana",m.getMana(),"items",m.getManaItems(),"auto_payment",m.isAutoPayment(),"restricted",m.isAutoPaymentRestricted(),"unlocked_type",m.getUnlockedManaType());}
        if(v instanceof Cost){Cost c=(Cost)v;JsonObject out=object("class",c.getClass().getName(),"id",c.getId(),"paid",c.isPaid(),"targets",c.getTargets());if(c instanceof ManaCost){ManaCost m=(ManaCost)c;out.add("mana",describe(m.getMana()));out.add("payment",describe(m.getPayment()));out.add("used",describe(m.getUsedManaToPay()));}return out;}
        if(v instanceof MageObject){MageObject o=(MageObject)v;return object("id",o.getId(),"name",o.getName(),"class",o.getClass().getName());}
        if(v instanceof Map){JsonArray a=new JsonArray();for(Object raw:((Map<?,?>)v).entrySet()){Map.Entry<?,?> e=(Map.Entry<?,?>)raw;a.add(object("key",e.getKey(),"value",e.getValue()));}return a;}
        if(v instanceof Iterable){JsonArray a=new JsonArray();for(Object x:(Iterable<?>)v)a.add(describe(x));return a;}
        if(v instanceof Object[]){JsonArray a=new JsonArray();for(Object x:(Object[])v)a.add(describe(x));return a;}
        
        throw new Stop("unsupported observation type: "+v.getClass().getName());
    }
    static Object targetCounters(Target target){
        for(Class<?> c=target.getClass();c!=Object.class;c=c.getSuperclass()){
            try {java.lang.reflect.Field f=c.getDeclaredField("zoneChangeCounters");f.setAccessible(true);return f.get(target);}
            catch(NoSuchFieldException e){/* inspect the pinned superclass */}
            catch(IllegalAccessException e){throw new Stop("unsupported target incarnation access");}
        }
        throw new Stop("unsupported target incarnation representation: "+target.getClass().getName());
    }
    static JsonObject textLabel(String text){
        if(text==null)return null;
        try{byte[] bytes=java.security.MessageDigest.getInstance("SHA-256").digest(text.getBytes(StandardCharsets.UTF_8));StringBuilder hex=new StringBuilder();for(byte b:bytes)hex.append(String.format("%02x",b&255));return object("text_sha256",hex.toString());}
        catch(Exception e){throw new Stop("text digest: "+e);}
    }
    // Source-native effect fields, including numeric parameters, target pointers,
    // timestamps and affected incarnation bindings. Text labels are not rules data.
    static JsonElement nativeFields(Object value,IdentityHashMap<Object,Integer> seen){
        if(value==null||value instanceof Number||value instanceof Boolean||value instanceof UUID||value instanceof Enum)return describe(value);
        if(value instanceof String)return textLabel((String)value);
        if(seen.containsKey(value))return object("reference",seen.get(value));
        int id=seen.size();seen.put(value,id);
        if(value instanceof Iterable){JsonArray a=new JsonArray();for(Object item:(Iterable<?>)value)a.add(nativeFields(item,seen));return object("node",id,"items",a);}
        if(value instanceof Map){JsonArray a=new JsonArray();for(Map.Entry<?,?> e:((Map<?,?>)value).entrySet())a.add(object("key",nativeFields(e.getKey(),seen),"value",nativeFields(e.getValue(),seen)));return object("node",id,"entries",a);}
        if(value.getClass().isArray()){JsonArray a=new JsonArray();for(int i=0;i<java.lang.reflect.Array.getLength(value);i++)a.add(nativeFields(java.lang.reflect.Array.get(value,i),seen));return object("node",id,"items",a);}
        if(!value.getClass().getName().startsWith("mage.")||value instanceof Game||value instanceof Player)throw new Stop("unsupported native field type: "+value.getClass().getName());
        JsonObject out=object("class",value.getClass().getName(),"node",id);
        for(Class<?> c=value.getClass();c!=Object.class;c=c.getSuperclass())for(java.lang.reflect.Field f:c.getDeclaredFields()){
            if(java.lang.reflect.Modifier.isStatic(f.getModifiers()))continue;
            try{f.setAccessible(true);out.add(c.getName()+"."+f.getName(),nativeFields(f.get(value),seen));}
            catch(IllegalAccessException e){throw new Stop("unsupported native field: "+e);}
        }
        return out;
    }
    static class Recorder implements AutoCloseable {
        Game live;
        final BufferedWriter writer;
        final Map<UUID,String> physical=new LinkedHashMap<>();
        long sequence;
        int calls, spells, attacks, simulationSuppressed;
        final int limit;
        String failure;
        boolean enabled=true;
        Recorder(Path path,int limit)throws IOException{writer=new BufferedWriter(new OutputStreamWriter(new java.util.zip.GZIPOutputStream(Files.newOutputStream(path,StandardOpenOption.CREATE_NEW)),StandardCharsets.UTF_8));this.limit=limit;}
        boolean accepts(Game g){if(g!=live||g.isSimulation()){simulationSuppressed++;return false;}return true;}
        void emit(String kind,UUID actor,Object detail,Game g){
            if(!accepts(g)||!enabled)return;
            if(sequence>=limit){failure="truncated: record limit";g.pause();throw new Stop(failure);}
            try{writer.write(JSON.toJson(object("sequence",sequence++,"simulation",g.isSimulation(),"kind",kind,"actor",actor,"detail",detail,"state",state(g))));writer.newLine();writer.flush();}
            catch(IOException e){failure="recording failure: "+e;throw new Stop(failure);}
        }
        void before(String method,UUID actor,Game g,Object... args){if(!accepts(g))return;calls++;emit("call/"+method,actor,args,g);}
        void after(String method,UUID actor,Game g,Object result,Object... args){if(!accepts(g))return;emit("return/"+method,actor,object("result",result,"arguments",args),g);}
        JsonObject state(Game g){
            JsonArray players=new JsonArray(),cards=new JsonArray(),permanents=new JsonArray(),stack=new JsonArray(),combat=new JsonArray(),effects=new JsonArray(),triggers=new JsonArray();
            for(Player p:g.getPlayers().values()){
                players.add(object("id",p.getId(),"life",p.getLife(),"hand",p.getHand(),"library",p.getLibrary().getCardList(),"graveyard",p.getGraveyard(),"mana",p.getManaPool().getMana(),"mana_native",p.getManaPool(),"lands_played",p.getLandsPlayed(),"empty_draw",p.getLibrary().isEmptyDraw(),"lost",p.hasLost(),"won",p.hasWon(),"quit",p.hasQuit(),"test_mode",p.isTestMode()));
                for(TriggeredAbility a:g.getState().getTriggered(p.getId()))triggers.add(describe(a));
            }
            for(Card c:g.getCards())cards.add(object("id",c.getId(),"physical",physical.get(c.getId()),"name",c.getName(),"owner",c.getOwnerId(),"zone",g.getState().getZone(c.getId()),"zcc",c.getZoneChangeCounter(g),"types",c.getCardType(g),"subtypes",c.getSubtype(g),"color",c.getColor(g).toString(),"power",c.getPower().getValue(),"toughness",c.getToughness().getValue(),"abilities",c.getAbilities(g)));
            for(Permanent p:g.getBattlefield().getAllPermanents()){
                if(p.isToken()&&!physical.containsKey(p.getId()))physical.put(p.getId(),"token/"+p.getId());
                permanents.add(object("id",p.getId(),"physical",physical.get(p.getId()),"name",p.getName(),"token",p.isToken(),"owner",p.getOwnerId(),"controller",p.getControllerId(),"zcc",p.getZoneChangeCounter(g),"tapped",p.isTapped(),"sick",p.hasSummoningSickness(),"damage",p.getDamage(),"power",p.getPower().getValue(),"toughness",p.getToughness().getValue(),"types",p.getCardType(g),"subtypes",p.getSubtype(g),"color",p.getColor(g).toString(),"abilities",p.getAbilities(g)));
            }
            for(StackObject o:g.getStack())stack.add(object("id",o.getId(),"source",o.getSourceId(),"controller",o.getControllerId(),"ability",o instanceof Spell?((Spell)o).getSpellAbility():((StackAbility)o).getStackAbility()));
            for(CombatGroup c:g.getCombat().getGroups())combat.add(object("attackers",c.getAttackers(),"blockers",c.getBlockers(),"defender",c.getDefenderId(),"blocked",c.getBlocked()));
            for(mage.abilities.effects.ContinuousEffect e:g.getContinuousEffects().getLayeredEffects(g))effects.add(object("id",e.getId(),"class",e.getClass().getName(),"duration",e.getDuration(),"abilities",g.getContinuousEffects().getLayeredEffectAbilities(e),"native_fields",nativeFields(e,new IdentityHashMap<>())));
            List<UUID> exile=new ArrayList<>();for(Card c:g.getExile().getCardsOwned(g,null))exile.add(c.getId());
            JsonObject point=object("turn",g.getTurnNum(),"step",g.getTurnStepType(),"active",g.getActivePlayerId(),"priority",g.getPriorityPlayerId(),"choosing",g.getState().getChoosingPlayerId(),"ended",g.hasEnded(),"players",players,"cards",cards,"permanents",permanents,"stack",stack,"combat",combat,"effects",effects,"pending_triggers",triggers,"exile",exile);
            return point;
        }
        public void close()throws IOException{writer.close();}
    }
    public static class CaptureGame extends TwoPlayerDuel {
        CaptureGame(){super(MultiplayerAttackOption.LEFT,RangeOfInfluence.ONE,new LondonMulligan(0),40,20,7);}
        CaptureGame(CaptureGame o){super(o);}
        @Override public CaptureGame copy(){return new CaptureGame(this);}
        @Override public void fireEvent(GameEvent e){
            if(recorder!=null&&recorder.accepts(this)){
                if(e.getType()==GameEvent.EventType.SPELL_CAST)recorder.spells++;
                recorder.emit("event/before",e.getPlayerId(),event(e),this);
            }
            super.fireEvent(e);
            if(recorder!=null)recorder.emit("event/after",e.getPlayerId(),event(e),this);
        }
        JsonObject event(GameEvent e){return object("type",e.getType(),"id",e.getId(),"source",e.getSourceId(),"target",e.getTargetId(),"amount",e.getAmount(),"data",e.getData()==null?null:textLabel(e.getData()),"zone",e.getZone(),"flag",e.getFlag(),"from",e instanceof ZoneChangeEvent?((ZoneChangeEvent)e).getFromZone():null,"to",e instanceof ZoneChangeEvent?((ZoneChangeEvent)e).getToZone():null,"source_ability",e instanceof ZoneChangeEvent?((ZoneChangeEvent)e).getSource():null);}
        @Override protected void resolve(){recorder.emit("resolution/before",null,null,this);super.resolve();recorder.emit("resolution/after",null,null,this);}
        @Override protected boolean checkStateBasedActions(){boolean changed=super.checkStateBasedActions();if(changed)recorder.emit("sba/after",null,null,this);return changed;}
    }
    public static class CapturePlayer extends ComputerPlayer7 {
        CapturePlayer(String name){super(name,RangeOfInfluence.ONE,1);}
        CapturePlayer(CapturePlayer o){super(o);}
        @Override public CapturePlayer copy(){return new CapturePlayer(this);}
        void detachControlCache(){actionCache=new HashSet<>(actionCache);maxDepth=4;maxNodes=5000;maxThinkTimeSecs=3;}
        void before(String method,Game game,Object... args){recorder.before(method,getId(),game,args);}
        void after(String method,Game game,Object result,Object... args){recorder.after(method,getId(),game,result,args);}
        @Override public boolean priority(Game game){
            if(firstPriority==null&&recorder.accepts(game)&&game.getTurnStepType()==PhaseStep.PRECOMBAT_MAIN){firstPriority=game.copy();detachControlCaches(firstPriority);firstActor=getId();}
            before("priority",game);
            boolean result=super.priority(game);
            after("priority",game,result);return result;
        }
        @Override public boolean playMana(Ability ability,ManaCost unpaid,String promptText,Game game){
            before("playMana",game,ability,unpaid,textLabel(promptText));
            boolean result=super.playMana(ability,unpaid,promptText,game);
            after("playMana",game,result,ability,unpaid,textLabel(promptText));return result;
        }
        @Override public void chooseRingBearer(Game game){
            if(recorder.accepts(game)){recorder.failure="unsupported callback: chooseRingBearer";game.pause();throw new Stop(recorder.failure);}
            super.chooseRingBearer(game);
        }
        // Generated explicit delegates follow; no tactical choices are replaced.
        @Override public void pass(Game game){
            before("pass",game);
            super.pass(game);
            after("pass",game,null);
        }
        @Override public void shuffleLibrary(Ability source, Game game){
            before("shuffleLibrary",game, source);
            super.shuffleLibrary(source, game);
            after("shuffleLibrary",game,null, source);
        }
        @Override public boolean cast(SpellAbility ability, Game game, boolean noMana, ApprovingObject approvingObject){
            before("cast",game, ability, noMana, approvingObject);
            boolean result=super.cast(ability, game, noMana, approvingObject);
            after("cast",game,result, ability, noMana, approvingObject);
            return result;
        }
        @Override public SpellAbility chooseAbilityForCast(Card card, Game game, boolean noMana){
            before("chooseAbilityForCast",game, card, noMana);
            SpellAbility result=super.chooseAbilityForCast(card, game, noMana);
            after("chooseAbilityForCast",game,result, card, noMana);
            return result;
        }
        @Override public ActivatedAbility chooseLandOrSpellAbility(Card card, Game game, boolean noMana){
            before("chooseLandOrSpellAbility",game, card, noMana);
            ActivatedAbility result=super.chooseLandOrSpellAbility(card, game, noMana);
            after("chooseLandOrSpellAbility",game,result, card, noMana);
            return result;
        }
        @Override public boolean playLand(Card card, Game game, boolean ignoreTiming){
            before("playLand",game, card, ignoreTiming);
            boolean result=super.playLand(card, game, ignoreTiming);
            after("playLand",game,result, card, ignoreTiming);
            return result;
        }
        @Override public boolean activateAbility(ActivatedAbility ability, Game game){
            before("activateAbility",game, ability);
            boolean result=super.activateAbility(ability, game);
            after("activateAbility",game,result, ability);
            return result;
        }
        @Override public void concede(Game game){
            before("concede",game);
            throw new Stop("unsupported callback: concession");
        }
        @Override public boolean choose(Outcome outcome, Target target, Ability source, Game game){
            before("choose",game, outcome, target, source);
            boolean result=super.choose(outcome, target, source, game);
            after("choose",game,result, outcome, target, source);
            return result;
        }
        @Override public boolean choose(Outcome outcome, Target target, Ability source, Game game, Map<String, Serializable> options){
            before("choose",game, outcome, target, source, options);
            boolean result=super.choose(outcome, target, source, game, options);
            after("choose",game,result, outcome, target, source, options);
            return result;
        }
        @Override public boolean choose(Outcome outcome, Cards cards, TargetCard target, Ability source, Game game){
            before("choose",game, outcome, cards, target, source);
            boolean result=super.choose(outcome, cards, target, source, game);
            after("choose",game,result, outcome, cards, target, source);
            return result;
        }
        @Override public boolean chooseTarget(Outcome outcome, Target target, Ability source, Game game){
            before("chooseTarget",game, outcome, target, source);
            boolean result=super.chooseTarget(outcome, target, source, game);
            after("chooseTarget",game,result, outcome, target, source);
            return result;
        }
        @Override public boolean chooseTarget(Outcome outcome, Cards cards, TargetCard target, Ability source, Game game){
            before("chooseTarget",game, outcome, cards, target, source);
            boolean result=super.chooseTarget(outcome, cards, target, source, game);
            after("chooseTarget",game,result, outcome, cards, target, source);
            return result;
        }
        @Override public boolean chooseTargetAmount(Outcome outcome, TargetAmount target, Ability source, Game game){
            before("chooseTargetAmount",game, outcome, target, source);
            boolean result=super.chooseTargetAmount(outcome, target, source, game);
            after("chooseTargetAmount",game,result, outcome, target, source);
            return result;
        }
        @Override public boolean chooseMulligan(Game game){
            before("chooseMulligan",game);
            boolean result=super.chooseMulligan(game);
            after("chooseMulligan",game,result);
            return result;
        }
        @Override public boolean chooseUse(Outcome outcome, String message, Ability source, Game game){
            before("chooseUse",game, outcome, textLabel(message), source);
            boolean result=super.chooseUse(outcome, message, source, game);
            after("chooseUse",game,result, outcome, textLabel(message), source);
            return result;
        }
        @Override public boolean chooseUse(Outcome outcome, String message, String secondMessage, String trueText, String falseText, Ability source, Game game){
            before("chooseUse",game, outcome, textLabel(message), textLabel(secondMessage), textLabel(trueText), textLabel(falseText), source);
            boolean result=super.chooseUse(outcome, message, secondMessage, trueText, falseText, source, game);
            after("chooseUse",game,result, outcome, textLabel(message), textLabel(secondMessage), textLabel(trueText), textLabel(falseText), source);
            return result;
        }
        @Override public boolean choose(Outcome outcome, Choice choice, Game game){
            before("choose",game, outcome, choice);
            boolean result=super.choose(outcome, choice, game);
            after("choose",game,result, outcome, choice);
            return result;
        }
        @Override public boolean choosePile(Outcome outcome, String message, List<? extends Card> pile1, List<? extends Card> pile2, Game game){
            before("choosePile",game, outcome, textLabel(message), pile1, pile2);
            boolean result=super.choosePile(outcome, message, pile1, pile2, game);
            after("choosePile",game,result, outcome, textLabel(message), pile1, pile2);
            return result;
        }
        @Override public int announceX(int min, int max, String message, Game game, Ability source, boolean isManaPay){
            before("announceX",game, min, max, textLabel(message), source, isManaPay);
            int result=super.announceX(min, max, message, game, source, isManaPay);
            after("announceX",game,result, min, max, textLabel(message), source, isManaPay);
            return result;
        }
        @Override public int chooseReplacementEffect(Map<String, String> effectsMap, Map<String, MageObject> objectsMap, Game game){
            before("chooseReplacementEffect",game, effectsMap, objectsMap);
            int result=super.chooseReplacementEffect(effectsMap, objectsMap, game);
            after("chooseReplacementEffect",game,result, effectsMap, objectsMap);
            return result;
        }
        @Override public TriggeredAbility chooseTriggeredAbility(List<TriggeredAbility> abilities, Game game){
            before("chooseTriggeredAbility",game, abilities);
            TriggeredAbility result=super.chooseTriggeredAbility(abilities, game);
            after("chooseTriggeredAbility",game,result, abilities);
            return result;
        }
        @Override public Mode chooseMode(Modes modes, Ability source, Game game){
            before("chooseMode",game, modes, source);
            Mode result=super.chooseMode(modes, source, game);
            after("chooseMode",game,result, modes, source);
            return result;
        }
        @Override public void selectAttackers(Game game, UUID attackingPlayerId){
            before("selectAttackers",game, attackingPlayerId);
            super.selectAttackers(game, attackingPlayerId);
            if(recorder.accepts(game)&&!game.getCombat().getGroups().isEmpty())recorder.attacks++;
            after("selectAttackers",game,null, attackingPlayerId);
        }
        @Override public void selectBlockers(Ability source, Game game, UUID defendingPlayerId){
            before("selectBlockers",game, source, defendingPlayerId);
            super.selectBlockers(source, game, defendingPlayerId);
            after("selectBlockers",game,null, source, defendingPlayerId);
        }
        @Override public int getAmount(int min, int max, String message, Ability source, Game game){
            before("getAmount",game, min, max, textLabel(message), source);
            int result=super.getAmount(min, max, message, source, game);
            after("getAmount",game,result, min, max, textLabel(message), source);
            return result;
        }
        @Override public List<Integer> getMultiAmountWithIndividualConstraints(Outcome outcome, List<MultiAmountMessage> messages, int totalMin, int totalMax, MultiAmountType type, Game game){
            before("getMultiAmountWithIndividualConstraints",game, outcome, messages, totalMin, totalMax, type);
            List<Integer> result=super.getMultiAmountWithIndividualConstraints(outcome, messages, totalMin, totalMax, type, game);
            after("getMultiAmountWithIndividualConstraints",game,result, outcome, messages, totalMin, totalMax, type);
            return result;
        }
    }
    static void detachControlCaches(Game game){
        for(Player p:game.getPlayers().values())((CapturePlayer)p).detachControlCache();
    }
    // Controlled same-input source-AI priority, with a genuine search in each run.
    // This does not claim unrestricted timed AI regeneration is deterministic.
    static JsonObject equivalence(Path folder,Game initial,UUID actor)throws Exception {
        if(initial==null)throw new Stop("missing controlled priority input");
        JsonObject[] states=new JsonObject[2];long[] records=new long[2];
        boolean[] choices=new boolean[2];
        for(int i=0;i<2;i++){
            Game g=initial.copy();detachControlCaches(g);
            try(Recorder r=new Recorder(folder.resolve("equivalence-"+i+".jsonl.gz"),100000)){
                recorder=r;r.live=g;r.enabled=i==1;
                // Same physical UUIDs, source state, seed, skill and search limits.
                RandomUtil.setSeed(319);
                choices[i]=g.getPlayer(actor).priority(g);
                states[i]=r.state(g);records[i]=r.sequence;
                Files.write(folder.resolve("equivalence-state-"+i+".json"),JSON.toJson(states[i]).getBytes(StandardCharsets.UTF_8),StandardOpenOption.CREATE_NEW);
                Game copy=g.copy();long before=r.sequence;r.emit("copy-probe",null,null,copy);
                if(!copy.getId().equals(g.getId())||r.sequence!=before)throw new Stop("search/snapshot copy escaped live identity guard");
            }
        }
        if(records[0]!=0||records[1]==0||choices[0]!=choices[1]||!states[0].equals(states[1]))throw new Stop("controlled recording-on/off choice/state difference");
        return object("kind","same-state-first-main-ComputerPlayer7-priority","rng_seed",319,"off_records",records[0],"on_records",records[1],"equal_return",true,"equal_semantic_state",true,"same_uuid_copy_excluded",true);
    }
    static JsonObject read(Path path)throws IOException{return JsonParser.parseString(new String(Files.readAllBytes(path),StandardCharsets.UTF_8)).getAsJsonObject();}
    static Card create(JsonObject metadata,UUID owner)throws Exception{
        JsonObject printing=metadata.getAsJsonObject("printing");
        for(mage.cards.ExpansionSet.SetCardInfo info:mage.sets.Foundations.getInstance().getSetCardInfo())
            if(info.getCardNumber().equals(printing.get("collector_number").getAsString())&&info.getName().equals(metadata.get("name").getAsString()))
                return (Card)info.getCardClass().getConstructor(UUID.class,CardSetInfo.class).newInstance(owner,new CardSetInfo(info.getName(),"FDN",info.getCardNumber(),info.getRarity(),info.getGraphicInfo()));
        throw new Stop("unsupported card printing: "+metadata.get("id"));
    }
    @Test public void capture()throws Exception{
        Path root=Paths.get(System.getProperty("mtglab.root")),folder=Paths.get(System.getProperty("mtglab.output"));
        JsonObject config=read(folder.resolve("config.json")),manifest=read(root.resolve("data/cards/foundations_micro_v1.json"));
        JsonObject result=object("status","failed","seed",config.get("seed"));
        try(Recorder r=new Recorder(folder.resolve("trace.jsonl.gz"),config.get("max_records").getAsInt())){
            recorder=r;firstPriority=null;firstActor=null;CaptureGame game=new CaptureGame();r.live=game;
            List<CapturePlayer> players=new ArrayList<>();
            for(int seat=0;seat<2;seat++){
                CapturePlayer p=new CapturePlayer("Seat"+seat);p.init(game);players.add(p);Deck deck=new Deck();
                String deckName=config.getAsJsonArray("decks").get(seat).getAsString();
                for(JsonElement raw:manifest.getAsJsonArray("decks"))if(raw.getAsJsonObject().get("id").getAsString().equals(deckName))
                    for(JsonElement entry:raw.getAsJsonObject().getAsJsonArray("cards")){
                        JsonObject c=entry.getAsJsonObject();String key=c.get("card_id").getAsString();JsonObject definition=null;
                        for(JsonElement d:manifest.getAsJsonArray("cards"))if(d.getAsJsonObject().get("id").getAsString().equals(key))definition=d.getAsJsonObject();
                        for(int copy=0;copy<c.get("copies").getAsInt();copy++){Card card=create(definition,p.getId());deck.getCards().add(card);r.physical.put(card.getId(),seat+"/"+key+"/"+copy);}
                    }
                if(deck.getCards().size()!=40)throw new Stop("unsupported frozen deck: "+deckName);
                p.setMatchPlayer(new mage.game.match.MatchPlayer(p,deck,new TwoPlayerMatch(new mage.game.match.MatchOptions("capture","Two Player Duel",false))));
                game.loadCards(deck.getCards(),p.getId());game.addPlayer(p,deck);
                if(p.isTestMode())throw new Stop("test-mode AI suppresses mulligans");
            }
            GameOptions options=new GameOptions();game.setGameOptions(options);
            if(options.testMode||options.skipInitShuffling)throw new Stop("non-normal game options");
            RandomUtil.setSeed(config.get("seed").getAsLong());
            UUID starter=players.get(config.get("starter").getAsInt()).getId();game.setStartingPlayerId(starter);
            r.emit("configuration",null,object("ai_class",ComputerPlayer7.class.getName(),"skill",1,"max_depth",4,"max_nodes",5000,"max_think_seconds",3,"rng","mage.util.RandomUtil/java.util.Random","test_mode",false,"skip_init_shuffling",false,"reference_variant","unmodified-upstream-raw","physical",r.physical,"config",config),game);
            try{
                if(config.get("unsupported_probe").getAsBoolean())players.get(0).chooseRingBearer(game);
                game.start(starter);
                if(r.failure!=null)throw new Stop(r.failure);
                if(!game.hasEnded())throw new Stop("truncated: game did not reach rules ending");
                for(Player p:game.getPlayers().values())if(p.hasQuit())throw new Stop("unsupported terminal: concession/quit");
                if(r.spells==0||r.attacks==0)throw new Stop("missing played spells or combat");
                JsonArray losses=new JsonArray();int winners=0;
                for(Player p:game.getPlayers().values()){
                    if(p.hasWon())winners++;
                    if(p.hasLost()){
                        boolean life=p.getLife()<=0,empty=p.getLibrary().isEmptyDraw();
                        if(!life&&!empty)throw new Stop("unsupported terminal reason");
                        losses.add(object("player",p.getId(),"life_loss",life,"empty_draw",empty));
                    }
                }
                if(losses.size()==0||winners>1)throw new Stop("unsupported terminal reason");
                result.add("terminal_reason",losses);
                r.emit("terminal",null,losses,game);result.addProperty("status","complete");
            }catch(Throwable e){result.addProperty("error",e.toString());result.addProperty("status",e.toString().contains("truncated")?"truncated":e.toString().contains("unsupported")?"unsupported":"failed");}
            if(result.get("status").getAsString().equals("complete")){
                try {result.add("equivalence",equivalence(folder,firstPriority,firstActor));}
                catch(Throwable e){result.addProperty("status","failed");result.addProperty("error","recording equivalence: "+e);}
                finally {recorder=r;}
            }
            result.addProperty("records",r.sequence);result.addProperty("calls",r.calls);result.addProperty("spells",r.spells);result.addProperty("attacks",r.attacks);result.addProperty("simulation_records_suppressed",r.simulationSuppressed);
        }catch(Throwable e){result.addProperty("error",e.toString());}
        Files.write(folder.resolve("result.json"),JSON.toJson(result).getBytes(StandardCharsets.UTF_8),StandardOpenOption.CREATE_NEW);
        if(!result.get("status").getAsString().equals("complete"))throw new AssertionError(JSON.toJson(result));
    }
}
