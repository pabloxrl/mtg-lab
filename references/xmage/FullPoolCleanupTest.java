package org.mage.test.mtglab;

import com.google.gson.*;
import mage.abilities.Ability;
import mage.abilities.costs.mana.ManaCost;
import mage.constants.*;
import mage.game.Game;
import mage.target.Target;
import mage.players.Player;
import org.mage.test.player.TestPlayer;
import org.junit.Test;
import java.util.*;
import java.nio.file.*;
import java.nio.charset.StandardCharsets;

/** Version 7: normal-reset cleanup and actual game-loop terminal observation. */
public class FullPoolCleanupTest extends FullPoolSpellsTest {
    private boolean cleanupObserverInstalled;
    private JsonObject beforeDeparture,beforeConcession;
    private JsonArray cleanupPoints;
    private Integer conceded;
    @Override protected void prepareCase(JsonObject c) {
        super.prepareCase(c);cleanupObserverInstalled=false;beforeDeparture=null;beforeConcession=null;cleanupPoints=new JsonArray();conceded=null;
    }
    private class Observer extends mage.watchers.Watcher {
        Observer(){super(WatcherScope.GAME);} Observer(Observer other){super(other);}
        @Override public mage.watchers.Watcher copy(){return new Observer(this);}
        @Override public void watch(mage.game.events.GameEvent e,Game g){
            if(e.getType()==mage.game.events.GameEvent.EventType.LOST)beforeDeparture=state(g,-1,"loss-before-departure",null);
            if(e.getType()==mage.game.events.GameEvent.EventType.CLEANUP_STEP_POST)cleanupPoints.add(state(g,-1,"cleanup-settled",null));
        }
    }
    @Override protected JsonObject result(){JsonObject r=super.result();r.add("before_departure",beforeDeparture);r.add("before_concession",beforeConcession);r.add("cleanup_checkpoints",cleanupPoints);return r;}

    @Override protected void validate(JsonObject doc) {
        need(integer(doc.get("schema_version"),7)&&doc.get("family").getAsString().equals("cleanup"),"/version/family");
        JsonObject projected=doc.deepCopy();projected.addProperty("schema_version",4);projected.addProperty("family","spells");
        for(JsonElement c:projected.getAsJsonArray("cases")) {
            need(Arrays.asList("prefix","terminal","concession").contains(c.getAsJsonObject().get("stop").getAsString()),"/completion kind");
            c.getAsJsonObject().addProperty("stop","resolved/unused");
            for(JsonElement raw:c.getAsJsonObject().getAsJsonArray("play")) {
                JsonObject e=raw.getAsJsonObject();if(e.get("kind").getAsString().equals("cleanup_discard")) {
                    e.remove("selection");e.addProperty("kind","discard");
                    for(String k:Arrays.asList("source","incarnation","color","role","mode"))e.add(k,JsonNull.INSTANCE);
                }
            }
        }
        super.validate(projected);
    }
    @Override protected JsonObject next(int actor,Game g) {
        need(cursor<current.getAsJsonArray("play").size(),"/omitted final checkpoint");
        JsonObject e=current.getAsJsonArray("play").get(cursor).getAsJsonObject();
        if(!e.get("kind").getAsString().equals("cleanup_discard"))return super.next(actor,g);
        keys(e,"sequence","turn","step","actor","kind","selection");
        need(integer(e.get("sequence"),cursor)&&integer(e.get("actor"),actor),"/play actor/sequence");
        need(integer(e.get("turn"),g.getTurnNum())&&e.get("step").getAsString().equals(g.getTurnStepType().name().toLowerCase(Locale.ROOT)),"/cleanup position");return e;
    }
    private JsonElement outcome(Game g) {
        if(!g.hasEnded())return JsonNull.INSTANCE;
        JsonObject out=new JsonObject();out.add("winner",JsonNull.INSTANCE);JsonArray losses=new JsonArray();
        for(int s=0;s<2;s++) {
            Player p=g.getPlayer(players[s].getId());if(p.hasWon())out.addProperty("winner",s);
            if(p.hasLost()) {
                if(conceded!=null && conceded==s)losses.add("Concession");else {need(p.getLibrary().isEmptyDraw(),"/unobserved loss reason");losses.add("EmptyDraw");}
            } else losses.add(JsonNull.INSTANCE);
        }
        out.add("losses",losses);return out;
    }
    @Override protected JsonObject state(Game g,int actor,String boundary,ManaCost unpaid) {
        JsonObject p=super.state(g,actor,boundary,unpaid);
        if(g.hasEnded() || actor<0)p.add("actor",JsonNull.INSTANCE);
        p.add("outcome",outcome(g));return p;
    }
    private void finish(Game g,int actor) {
        need(cursor<current.getAsJsonArray("play").size(),"/omitted final checkpoint");
        JsonObject e=current.getAsJsonArray("play").get(cursor).getAsJsonObject();
        need(e.get("kind").getAsString().equals("finish"),"/choice after terminal");
        for(String k:Arrays.asList("source","incarnation","color","role","mode"))need(e.get(k).isJsonNull(),"/finish unexpected choice");
        need(integer(e.get("sequence"),cursor),"/sequence");
        need(cursor+1==current.getAsJsonArray("play").size(),"/unused tape suffix");
        need(integer(e.get("turn"),g.getTurnNum())&&e.get("step").getAsString().equals(g.getTurnStepType().name().toLowerCase(Locale.ROOT)),"/final checkpoint position");
        need((current.get("stop").getAsString().equals("terminal")||current.get("stop").getAsString().equals("concession"))==g.hasEnded(),"/premature terminal");
        if(g.hasEnded())need(current.get("stop").getAsString().equals("concession")== (conceded!=null),"/concession completion classification");
        played.add(state(g,actor,"finish",null));accept(e);stopped=true;
    }
    @Override protected boolean onPriority(TestPlayer p,int s,Game g) {
        if(!cleanupObserverInstalled){g.getState().addWatcher(new Observer());cleanupObserverInstalled=true;}

        if(cursor<current.getAsJsonArray("play").size() && current.getAsJsonArray("play").get(cursor).getAsJsonObject().get("kind").getAsString().equals("concede")) {
            try {
                if(!openingSeen){firstUpkeep(p,s,g);openingSeen=true;}
                JsonObject e=next(s,g);played.add(state(g,s,"before/"+cursor,null));beforeConcession=state(g,s,"before-concession",null);conceded=s;accept(e);p.concede(g);return false;
            }catch(IllegalArgumentException e){failCallback(g,e);return false;}
        }
        if(cursor<current.getAsJsonArray("play").size()&&current.getAsJsonArray("play").get(cursor).getAsJsonObject().get("kind").getAsString().equals("finish")) {
            try {finish(g,s);g.pause();}catch(IllegalArgumentException e){failCallback(g,e);}return false;
        }
        return super.onPriority(p,s,g);
    }
    @Override protected void afterGame(Game g) {
        if(g.hasEnded()){
            try {finish(g,-1);need(observed.size()>0,"/missing opening observations");}
            catch(IllegalArgumentException error){throw new IllegalArgumentException("first divergence: /play/"+cursor+" "+error.getMessage().replace("first divergence: ",""));}
        }
        else super.afterGame(g);
    }
    @Override protected boolean onSpellTarget(TestPlayer p,int s,Outcome o,Target target,Ability a,Game g) {
        if(g.getTurnStepType()!=PhaseStep.CLEANUP || a!=null)return super.onSpellTarget(p,s,o,target,a,g);
        try {
            need(!stopped&&o==Outcome.Discard&&target!=null,"/unexpected cleanup callback");
            JsonObject e=next(s,g);need(e.get("kind").getAsString().equals("cleanup_discard"),"/missing cleanup discard");
            JsonArray selected=e.getAsJsonArray("selection");
            need(selected.size()==target.getMinNumberOfTargets()&&selected.size()==target.getMaxNumberOfTargets(),"/cleanup cardinality");
            played.add(state(g,s,"before/"+cursor,null));
            for(JsonElement raw:selected) {
                JsonObject card=raw.getAsJsonObject();keys(card,"source","incarnation");UUID id=source(card,g);
                need(p.getHand().contains(id)&&!target.getTargets().contains(id)&&target.canTarget(p.getId(),id,a,g),"/cleanup discard source");
                target.addTarget(id,a,g);need(target.getTargets().contains(id),"/cleanup discard rejected");
            }
            need(target.getTargets().size()==selected.size(),"/cleanup discard cardinality");accept(e);return true;
        }catch(IllegalArgumentException e){failCallback(g,e);return false;}
    }
    @Override @Test public void mulliganPrefixes() throws Exception {
        JsonObject doc=read(Paths.get(System.getProperty("mtglab.fixture")));validate(doc);
        JsonObject points=new JsonObject(),runs=new JsonObject(),repeats=new JsonObject();
        for(JsonElement element:doc.getAsJsonArray("cases")) {
            JsonObject c=element.getAsJsonObject();String id=c.get("id").getAsString();JsonObject r=run(c);points.add(id,r.get("points"));runs.add(id,r);repeats.add(id,run(c));
        }
        JsonObject rejections=new JsonObject(),negativeRuns=new JsonObject();
        JsonObject negatives=read(root.resolve("fixtures/reference/full-pool-cleanup-negatives.json"));
        for(String name:negatives.keySet()) {
            JsonObject spec=negatives.getAsJsonObject(name);
            try {JsonObject d=spec.getAsJsonObject("input");validate(d);run(d.getAsJsonArray("cases").get(0).getAsJsonObject());org.junit.Assert.fail("accepted negative "+name);}
            catch(IllegalArgumentException error){
                String expected="first divergence: /play/"+spec.get("sequence").getAsInt()+" "+spec.get("xmage").getAsString();
                // afterGame finalization runs outside the priority callback.
                need(error.getMessage().equals(expected),"/intended rejection "+name+": "+error.getMessage());
                rejections.addProperty(name,error.getMessage());negativeRuns.add(name,read(Paths.get(System.getProperty("mtglab.output")+".failure.json")));
            }
        }
        JsonObject out=new JsonObject();out.add("checkpoints",points);out.add("runs",runs);out.add("repeat_runs",repeats);out.add("rejections",rejections);out.add("negative_runs",negativeRuns);
        Files.write(Paths.get(System.getProperty("mtglab.output")),JSON.toJson(out).getBytes(StandardCharsets.UTF_8));
    }
}
