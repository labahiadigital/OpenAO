import { PacketWriter, SERVER_PACKET_ID } from "@openao/protocol";
import { gameSession } from "./gameSession.svelte";
import { gameState } from "../state/gameState.svelte";

export function sendPosition(heading: number, moveId: number) {
  const writer = new PacketWriter(SERVER_PACKET_ID.position);
  writer.writeByte(heading);
  writer.writeShort(moveId);

  const packet = gameState.inputSender.packet(0);
  if (packet && packet.inputs.length > 1) {
    const redundant = packet.inputs.slice(0, -1);
    writer.writeByte(redundant.length);
    for (const frame of redundant) {
      writer.writeShort(frame.sequence);
      writer.writeByte(frame.input.heading);
    }
  } else {
    writer.writeByte(0);
  }

  gameSession.send(writer.toArrayBuffer());
}

export function sendChangeHeading(heading: number) {
  const writer = new PacketWriter(SERVER_PACKET_ID.changeHeading);
  writer.writeByte(heading);
  gameSession.send(writer.toArrayBuffer());
}

export function sendAttackMelee() {
  const writer = new PacketWriter(SERVER_PACKET_ID.attackMele);
  gameSession.send(writer.toArrayBuffer());
}

export function sendAttackRange() {
  const writer = new PacketWriter(SERVER_PACKET_ID.attackRange);
  gameSession.send(writer.toArrayBuffer());
}

export function sendAttackSpell(spellSlot: number) {
  const writer = new PacketWriter(SERVER_PACKET_ID.attackSpell);
  writer.writeByte(spellSlot);
  gameSession.send(writer.toArrayBuffer());
}

export function sendDialog(message: string) {
  const writer = new PacketWriter(SERVER_PACKET_ID.dialog);
  writer.writeString(message);
  gameSession.send(writer.toArrayBuffer());
}

export function sendEquipItem(slot: number) {
  const writer = new PacketWriter(SERVER_PACKET_ID.equiparItem);
  writer.writeByte(slot);
  gameSession.send(writer.toArrayBuffer());
}

export function sendUseItemClick(slot: number) {
  const writer = new PacketWriter(SERVER_PACKET_ID.useItemClick);
  writer.writeByte(slot);
  gameSession.send(writer.toArrayBuffer());
}

export function sendUseItemU(slot: number) {
  const writer = new PacketWriter(SERVER_PACKET_ID.useItemU);
  writer.writeByte(slot);
  gameSession.send(writer.toArrayBuffer());
}

export function sendDropItem(slot: number, quantity: number) {
  const writer = new PacketWriter(SERVER_PACKET_ID.tirarItem);
  writer.writeByte(slot);
  writer.writeShort(quantity);
  gameSession.send(writer.toArrayBuffer());
}

export function sendPickupItem() {
  const writer = new PacketWriter(SERVER_PACKET_ID.agarrarItem);
  gameSession.send(writer.toArrayBuffer());
}

export function sendToggleSafe() {
  const writer = new PacketWriter(SERVER_PACKET_ID.changeSeguro);
  gameSession.send(writer.toArrayBuffer());
}

export function sendResyncPosition() {
  const writer = new PacketWriter(SERVER_PACKET_ID.resyncPosition);
  gameSession.send(writer.toArrayBuffer());
}

export function sendCraftItem(recipeIndex: number) {
  const writer = new PacketWriter(SERVER_PACKET_ID.craftItem);
  writer.writeShort(recipeIndex);
  gameSession.send(writer.toArrayBuffer());
}

export function sendCloseTrade() {
  const writer = new PacketWriter(SERVER_PACKET_ID.closeTrade);
  gameSession.send(writer.toArrayBuffer());
}

export function sendBuyItem(npcSlot: number, amount: number) {
  const writer = new PacketWriter(SERVER_PACKET_ID.buyItem);
  writer.writeByte(npcSlot);
  writer.writeShort(amount);
  gameSession.send(writer.toArrayBuffer());
}

export function sendSellItem(invSlot: number, amount: number) {
  const writer = new PacketWriter(SERVER_PACKET_ID.sellItem);
  writer.writeByte(invSlot);
  writer.writeShort(amount);
  gameSession.send(writer.toArrayBuffer());
}

export function sendClick(x: number, y: number) {
  const writer = new PacketWriter(SERVER_PACKET_ID.click);
  writer.writeShort(x);
  writer.writeShort(y);
  gameSession.send(writer.toArrayBuffer());
}

export function sendPing(token: number) {
  const writer = new PacketWriter(SERVER_PACKET_ID.ping);
  writer.writeInt(token);
  gameSession.send(writer.toArrayBuffer());
}

export function sendMarketAction(action: string, payload: Record<string, unknown> = {}) {
  const writer = new PacketWriter(SERVER_PACKET_ID.marketAction);
  writer.writeString(JSON.stringify({ action, ...payload }));
  gameSession.send(writer.toArrayBuffer());
}

export function sendRetosAction(action: string, payload: Record<string, unknown> = {}) {
  const writer = new PacketWriter(SERVER_PACKET_ID.retosAction);
  writer.writeString(JSON.stringify({ action, ...payload }));
  gameSession.send(writer.toArrayBuffer());
}

export function sendReorderInventory(source: number, target: number) {
  const writer = new PacketWriter(SERVER_PACKET_ID.reorderInventoryItem);
  writer.writeByte(source);
  writer.writeByte(target);
  gameSession.send(writer.toArrayBuffer());
}

export function sendReorderSpell(source: number, target: number) {
  const writer = new PacketWriter(SERVER_PACKET_ID.reorderSpell);
  writer.writeByte(source);
  writer.writeByte(target);
  gameSession.send(writer.toArrayBuffer());
}

export function sendDepositBankGold(amount: number) {
  const writer = new PacketWriter(SERVER_PACKET_ID.depositBankGold);
  writer.writeInt(amount);
  gameSession.send(writer.toArrayBuffer());
}

export function sendWithdrawBankGold(amount: number) {
  const writer = new PacketWriter(SERVER_PACKET_ID.withdrawBankGold);
  writer.writeInt(amount);
  gameSession.send(writer.toArrayBuffer());
}
