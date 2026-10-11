package org.mage.test.mtglab;

import mage.constants.*;
import mage.game.*;
import mage.game.events.*;
import mage.game.mulligan.LondonMulligan;
import mage.game.permanent.Permanent;
import mage.game.permanent.PermanentToken;
import java.util.*;

/** Scoped, explicitly patched reference semantics for CR101.4/404.3/704.3.
 * Upstream source is unchanged. Only combat lethal-creature SBA batching is
 * replaced. ZonesHandler removes every source before placing any destination
 * and queues simultaneous zone events. No graveyard is reordered afterward.
 * Other mechanics remain upstream; unsupported replacement behavior fails.
 */
public final class CombatSbaGame extends TwoPlayerDuel {
    public interface OwnerChoice {
        List<Permanent> choose(UUID owner,List<Permanent> arrivals,Game game);
        void failure(Game game,IllegalArgumentException error);
    }
    private final OwnerChoice choices;
    public CombatSbaGame(OwnerChoice choices){
        super(MultiplayerAttackOption.LEFT,RangeOfInfluence.ONE,new LondonMulligan(0),40,20,7);
        this.choices=choices;
    }
    private CombatSbaGame(CombatSbaGame original){super(original);choices=original.choices;}
    @Override public CombatSbaGame copy(){return new CombatSbaGame(this);}
    private static void need(boolean ok,String message){if(!ok)throw new IllegalArgumentException("first divergence: "+message);}
    @Override protected boolean checkStateBasedActions(){
        try{return combatSba();}catch(IllegalArgumentException error){choices.failure(this,error);return false;}
    }
    private boolean combatSba(){
        if(getTurnStepType()!=PhaseStep.COMBAT_DAMAGE)return super.checkStateBasedActions();
        List<Permanent> dying=new ArrayList<>();
        for(Permanent p:getBattlefield().getAllActivePermanents())if(p.isCreature(this)){
            need(getState().getActivePowerInsteadOfToughnessForDamageLethalityFilters().isEmpty(),"/unsupported lethal replacement");
            if(p.getToughness().getValue()<=0 || p.getDamage()>=p.getToughness().getValue() || p.isDeathtouched())dying.add(p);
        }
        if(dying.isEmpty())return super.checkStateBasedActions();
        for(Permanent p:dying)if(p.getToughness().getValue()>0){
            need(!p.getAbilities().containsKey(mage.abilities.keyword.IndestructibleAbility.getInstance().getId()),"/unsupported indestructible");
            need(!replaceEvent(GameEvent.getEvent(GameEvent.EventType.DESTROY_PERMANENT,p.getId(),null,p.getControllerId())),"/unsupported destruction replacement");
        }
        List<Permanent> ordered=new ArrayList<>();
        List<UUID> owners=new ArrayList<>();owners.add(getActivePlayerId());
        for(UUID id:getPlayers().keySet())if(!id.equals(getActivePlayerId()))owners.add(id);
        for(UUID owner:owners){
            List<Permanent> arrivals=new ArrayList<>();for(Permanent p:dying)if(p.getOwnerId().equals(owner)&&!(p instanceof PermanentToken))arrivals.add(p);
            if(arrivals.size()>1){
                List<Permanent> selected=choices.choose(owner,Collections.unmodifiableList(arrivals),this);
                need(selected.size()==arrivals.size()&&new HashSet<>(selected).size()==arrivals.size()&&selected.containsAll(arrivals),"/graveyard permutation");
                arrivals=selected;
            }
            ordered.addAll(arrivals);
            // CR111.6: tokens are not cards in the owner permutation.
            for(Permanent p:dying)if(p.getOwnerId().equals(owner)&&p instanceof PermanentToken)ordered.add(p);
        }
        List<ZoneChangeInfo> moves=new ArrayList<>();
        for(Permanent p:ordered)moves.add(new ZoneChangeInfo(new ZoneChangeEvent(p,null,p.getOwnerId(),Zone.BATTLEFIELD,Zone.GRAVEYARD)));
        List<ZoneChangeInfo> moved=ZonesHandler.moveCards(moves,null,this);
        need(moved.size()==dying.size(),"/unsupported zone replacement");
        for(Permanent p:ordered)if(p.getToughness().getValue()>0)addSimultaneousEvent(GameEvent.getEvent(GameEvent.EventType.DESTROYED_PERMANENT,p.getId(),null,p.getControllerId()));
        super.checkStateBasedActions();
        return true;
    }
}
