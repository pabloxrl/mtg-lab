package forge.game;

import com.google.gson.*;
import forge.game.card.*;
import forge.game.event.Event;
import forge.game.player.Player;
import forge.game.zone.ZoneType;
import java.nio.file.*;
import java.util.*;

/** Test-only strict prefix controller. Forge alone applies the game rules. */
public final class MtgOpeningReplay {
    private static MtgOpeningReplay active;
    private final Game game;
    private final JsonObject tape;
    private final JsonArray rows;
    private int cursor;
    private int randoms;
    private int decisions;
    public static final class Endpoint extends RuntimeException { private Endpoint() { super("opening prefix endpoint"); } }
    private MtgOpeningReplay(Game g, JsonObject t) { game=g; tape=t; rows=t.getAsJsonArray("prefix"); }
    public static void open(Game g, Path file) throws Exception {
        if (active != null) throw new IllegalStateException("replay already open");
        active=new MtgOpeningReplay(g, JsonParser.parseString(Files.readString(file)).getAsJsonObject());
        active.take("start");
    }
    private void require(boolean ok, String field) {
        if (!ok) throw new IllegalArgumentException("first divergence: /forge/"+cursor+"/"+field);
    }
    private static JsonElement inspect(Game game, Object value) {
        try {
            java.lang.reflect.Constructor<MtgRawCapture> constructor=MtgRawCapture.class.getDeclaredConstructor(Game.class,Path.class,long.class);
            constructor.setAccessible(true);
            MtgRawCapture observer=constructor.newInstance(game,null,Long.MAX_VALUE);
            java.lang.reflect.Method method=MtgRawCapture.class.getDeclaredMethod("value",Object.class);
            method.setAccessible(true);
            return (JsonElement)method.invoke(observer,value);
        } catch (ReflectiveOperationException error) { throw new IllegalStateException("capture value observer unavailable",error); }
    }
    private static JsonElement semantic(JsonElement e) {
        if (e.isJsonArray()) { JsonArray a=new JsonArray(); for(JsonElement v:e.getAsJsonArray()) a.add(semantic(v)); return a; }
        if (e.isJsonObject()) {
            JsonObject o=new JsonObject();
            for(Map.Entry<String,JsonElement> v:e.getAsJsonObject().entrySet())
                if (!v.getKey().equals("instance")) o.add(v.getKey(),semantic(v.getValue()));
            return o;
        }
        return e;
    }
    private static String difference(JsonElement expected,JsonElement actual,String path) {
        if(expected.equals(actual))return null;
        if(expected.isJsonObject() && actual.isJsonObject()) {
            JsonObject e=expected.getAsJsonObject(),a=actual.getAsJsonObject();
            if(!e.keySet().equals(a.keySet()))return path+"/fields";
            for(String key:e.keySet()) { String d=difference(e.get(key),a.get(key),path+"/"+key); if(d!=null)return d; }
        }
        if(expected.isJsonArray() && actual.isJsonArray()) {
            JsonArray e=expected.getAsJsonArray(),a=actual.getAsJsonArray();
            if(e.size()!=a.size())return path+"/length";
            for(int i=0;i<e.size();i++) { String d=difference(e.get(i),a.get(i),path+"/"+i); if(d!=null)return d; }
        }
        return path;
    }
    private void state(JsonObject row) {
        JsonElement expected=semantic(row.get("state")), actual=semantic(MtgRawCapture.audit(game));
        String difference=difference(expected,actual,"");
        require(difference==null,"state"+difference);
    }
    private JsonObject take(String kind) {
        require(cursor<rows.size(),"unrecorded "+kind);
        JsonObject row=rows.get(cursor).getAsJsonObject();
        require(row.get("index").getAsInt()==cursor && row.get("kind").getAsString().equals(kind),"record kind "+kind);
        if (!kind.equals("random")) state(row);
        cursor++;
        return row;
    }
    public static void event(Game g, Event e) {
        if(active==null)return;
        JsonObject row=active.take("event");
        active.require(g==active.game && row.get("event").getAsString().equals(e.getClass().getSimpleName()),"event");
    }
    public static void boundary(Game g,String name) {
        if(active==null)return;
        JsonObject row=active.take("boundary");
        active.require(g==active.game && row.get("boundary").getAsString().equals(name),"boundary");
    }
    public static void shuffle(Player p,String stage) {
        if(active==null)return;
        JsonObject row=active.take("shuffle");
        active.require(row.get("stage").getAsString().equals(stage) && row.getAsJsonObject("actor").get("player_id").getAsInt()==p.getId(),"shuffle actor/stage");
        active.require(semantic(row.get("library")).equals(semantic(inspect(p.getGame(),p.getCardsIn(ZoneType.Library)))),"shuffle order");
    }
    public static final class RecordedRandom extends Random {
        public RecordedRandom() { super(0); }
        @Override protected int next(int bits) {
            if(active==null) throw new IllegalStateException("random outside opening replay");
            JsonObject r=active.take("random");
            active.require(r.get("bits").getAsInt()==bits && r.get("draw").getAsInt()==++active.randoms,"random bits/order");
            return r.get("result").getAsInt();
        }
    }
    @SuppressWarnings("unchecked")
    public static <T> T choose(Player p,String method,Object[] args) {
        if(active==null)throw new IllegalStateException("choice outside opening replay: "+method);
        MtgOpeningReplay a=active;
        if(a.cursor==a.rows.size()) {
            JsonObject endpoint=a.tape.getAsJsonObject("endpoint");
            a.require(method.equals("chooseSpellAbilityToPlay") && method.equals(endpoint.get("method").getAsString()) &&
                endpoint.getAsJsonObject("actor").get("player_id").getAsInt()==p.getId(),"unconsumed opening choice/endpoint");
            a.state(endpoint);
            throw new Endpoint();
        }
        JsonObject entry=a.take("enter");
        a.require(entry.get("method").getAsString().equals(method) && entry.getAsJsonObject("actor").get("player_id").getAsInt()==p.getId(),"callback actor/method");
        a.require(semantic(entry.get("arguments")).equals(semantic(inspect(p.getGame(),Arrays.asList(args)))),"callback arguments");
        int end=a.tape.getAsJsonObject("paired").get(entry.get("index").getAsString()).getAsInt();
        // Internal AI deliberation is retained and accounted for, never rerun.
        // The source parser pairs it; no rules transition may be hidden here.
        while(a.cursor<end) {
            JsonObject inner=a.rows.get(a.cursor).getAsJsonObject();
            String kind=inner.get("kind").getAsString();
            a.require(Set.of("enter","leave","random").contains(kind),"rules event inside AI choice");
            inner=a.take(kind);
            if(kind.equals("random")) a.require(inner.get("draw").getAsInt()==++a.randoms,"internal random order");
        }
        JsonObject leave=a.take("leave");
        a.require(leave.get("call").equals(entry.get("call")),"callback return");
        JsonElement result=leave.get("result");
        a.decisions++;
        Object answer;
        switch(method) {
            case "mulliganKeepHand": answer=result.getAsBoolean(); break;
            case "chooseStartingPlayer":
                answer=p.getGame().getPlayers().stream().filter(q->q.getId()==result.getAsJsonObject().get("player_id").getAsInt()).findFirst().orElseThrow(); break;
            case "cheatShuffle": case "tuckCardsViaMulligan":
                CardCollection cards=new CardCollection(); Set<Integer> ids=new HashSet<>();
                CardCollectionView available=(CardCollectionView)args[0];
                for(JsonElement e:result.getAsJsonArray()) {
                    JsonObject c=e.getAsJsonObject(); int id=c.get("id").getAsInt();
                    a.require(ids.add(id),"duplicate occurrence");
                    Card card=null;
                    for(Card candidate:available) if(candidate.getId()==id) card=candidate;
                    a.require(card!=null && card.getGameTimestamp()==c.get("timestamp").getAsLong(),"stale occurrence/incarnation");
                    cards.add(card);
                }
                a.require(method.equals("cheatShuffle") ? cards.size()==available.size() : cards.size()==((Number)args[1]).intValue(),"choice cardinality");
                answer=cards; break;
            case "complainCardsCantPlayWell": a.require(result.isJsonArray() && result.getAsJsonArray().isEmpty(),"unsupported complaint"); answer=Collections.emptyMap(); break;
            case "awaitNextInput": case "revealUnsupported": a.require(result.isJsonNull(),"void result"); answer=null; break;
            default: throw new IllegalArgumentException("unsupported opening choice: "+method);
        }
        return (T)answer;
    }
    public static void finish(Path output) throws Exception {
        MtgOpeningReplay a=active;
        a.require(a.cursor==a.rows.size() && !a.game.isGameOver(),"prefix consumption/terminal");
        JsonObject result=new JsonObject();
        result.addProperty("status","opening-replayed"); result.addProperty("complete_game",false);
        result.addProperty("consumed_records",a.cursor); result.addProperty("consumed_random",a.randoms);
        result.addProperty("controller_choices",a.decisions); result.add("endpoint",a.tape.getAsJsonObject("endpoint").get("index"));
        result.add("remainder",a.tape.get("remainder")); result.add("state",MtgRawCapture.audit(a.game));
        Files.writeString(output,result.toString(),StandardOpenOption.CREATE_NEW); active=null;
    }
}
