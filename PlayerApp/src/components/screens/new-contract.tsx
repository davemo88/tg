import React, { useEffect, useState, } from 'react';
import { useDispatch } from 'react-redux';
import { Alert, Modal, Image, Button, StyleSheet, Text, TextInput, View, } from 'react-native';

import { styles } from '../../styles';

import { store, playerSlice, playerSelectors, contractSelectors, contractSlice, selectedPlayerNameSlice, balanceSlice, newContract, } from '../../redux';
import { isEvent, Event, Player, Contract, ContractStatus } from '../../datatypes';
import { getPosted } from '../../wallet';

import { Secret } from '../../secret';
import { Currency } from '../currency';
import { PlayerPortrait } from '../player-portrait';
import { PlayerSelector } from '../player-selector';

// TODO: add referee / game-domain expertise delegate pubkey input
export const NewContract = ({ navigation }) => {
    const dispatch = useDispatch();
    const selectedPlayer = playerSelectors.selectById(store.getState(), store.getState().selectedPlayerName);
    const playerTwos = playerSelectors
        .selectAll(store.getState())
        .filter((player: Player, i, a) => !player.mine);
    const [contractAmount, onChangeContractAmount] = React.useState('0');
    const [playerTwoName, setPlayerTwoName] = useState(playerTwos.length > 0 ? playerTwos[0].name : null);
    const [playerTwoPosted, setPlayerTwoPosted] = useState(0);
    const [creatingContract, setCreatingContract] = useState(false);
    const [eventModalVisible, setEventModalVisible] = useState(false);

    const [eventText, setEventText] = useState('');
    const [eventTextError, setEventTextError] = useState<string|null>(null);
    const [event, setEvent] = useState<Event|null>(null);

    const [eventPayouts, setEventPayouts] = useState<string[]>([]);

    const amountValid = (): boolean => {
        if (parseInt(contractAmount) > 0) {
            return true
        }
        return false
    }

    const eventValid = (): boolean => {
        try {
            let maybeEvent = JSON.parse(eventText);
            if (isEvent(maybeEvent)) {
                return true;
            }
        } catch(e) {
            console.error(Error(e));
        }
        return false;
    }

    const setPlayerTwo = (p2Name: string) => {
        if (playerTwoName!== null) {
            let p2Idx = eventPayouts.indexOf(playerTwoName);
            if (p2Idx !== -1) {
                eventPayouts[p2Idx] = p2Name;
            }
        }
        setPlayerTwoName(p2Name);
    }

    useEffect(() => {
//TODO: could cache these to limit unneccesary network calls
        if (selectedPlayer && playerTwoName) {
            getPosted(playerTwoName)
                .then((posted) => setPlayerTwoPosted(posted),
                )
                .catch(error => console.error(error));
        }
    }, [playerTwoName]);

    return (
        <View style={styles.container}>
            <Modal
                animationType="slide"
                transparent={true}
                visible={eventModalVisible}
                onRequestClose={() => {
                    Alert.alert("event modal closed");
                    setEventModalVisible(!eventModalVisible);
                }}
            >
                <View style={{
                    flex: 1,
                    justifyContent: "center",
                    alignItems: "center",
                    marginTop: 22,
                }}>
                    <View style={{
                        backgroundColor: "white",
                        borderRadius: 20,
                        alignItems: "center",
                        shadowColor: "#000",
                        shadowOffset: {
                          width: 0,
                          height: 2
                        },
                        shadowOpacity: 0.25,
                        shadowRadius: 4,
                        elevation: 5
                    }}>
                        <View style={{ backgroundColor: 'lightslategrey', alignItems: 'center', padding: 10, margin: 10 }}>
                            <Text>Paste Event JSON Below</Text>
                            <TextInput 
                                value={eventText}
                                onChangeText={(text) => {
                                    setEventText(text);
                                }}
                                style={{ borderWidth: 1, width: 100, margin: 10, padding: 4 }}
                            />
                        </View>
                        {
                            eventTextError !== null && 
                            <Text>{eventTextError}</Text>
                        }
                        <View style={{ margin: 10, flexDirection: 'row' }}>
                            <View style={{ margin: 3 }}>
                                <Button 
                                    title="Submit" 
                                    onPress={() => { 
                                        setEventTextError(null);
                                        if (eventValid()) {
                                            setEvent(JSON.parse(eventText));
                                            if (playerTwoName !== null) {
                                                setEventPayouts([selectedPlayer.name, playerTwoName]);
                                            }
                                            setEventModalVisible(!eventModalVisible)
                                        } else {
                                            setEventTextError("invalid event JSON");
                                        }
                                    }}
                                />
                            </View>
                            <View style={{ margin: 3 }}>
                                <Button 
                                    title="Clear" 
                                    onPress={() => {
                                        setEventText('');
                                        setEventTextError(null);
                                        setEvent(null);
                                    }}
                                />
                            </View>
                            <View style={{ margin: 3 }}>
                                <Button 
                                    title="Cancel" 
                                    onPress={() => setEventModalVisible(!eventModalVisible)}
                                />
                            </View>
                        </View>
                    </View>
                </View>
            </Modal>
            <Text style={{ fontSize: 20 }}>Choose Player</Text>
            { playerTwoName !== null ? 
                <PlayerSelector selectedPlayerName={playerTwoName} setSelectedPlayerName={setPlayerTwo} playerNames={playerTwos.map((p: Player) => p.name)} allowRemoval={true} />
                : <Text>No Players</Text>
            }
            <Text>Posted Player Balance</Text>
            <Currency amount={playerTwoPosted} />
            <View style={{ margin: 10 }}>
                <Button 
                    title="Add Player" 
                    onPress={() => navigation.navigate('Add Player') }
                />
            </View>
            <View style={{ alignItems: "center", margin: 10 }}>
                <View style={{ maxWidth: 150 }}>
                    <Button
                        title="Use Event" 
                        onPress={() => setEventModalVisible(!eventModalVisible)}
                    />
                </View>
                {
                    event !== null &&
                    <EventInfo event={event} payouts={eventPayouts} setEventPayouts={setEventPayouts}/>
                }
            </View>
            <View style={{ backgroundColor: 'lightslategrey', alignItems: 'center', padding: 10, flexDirection: 'row' }}>
                <Text style={{ fontSize: 16 }}>Amount</Text>
                <View style={{ flexDirection: 'row', alignItems: 'center', backgroundColor: 'lightslategrey', }}>
                    <TextInput
                        onChangeText={text => onChangeContractAmount(text)}
                        onBlur={() => {if (Number.isNaN(parseInt(contractAmount))) { onChangeContractAmount('0')}}}
                        value={contractAmount}
                        style={{ borderWidth: 1, width: 100, margin: 10, padding: 4, textAlign: 'right' }}
                    />         
                </View>
            </View>
            <View style={{ flexDirection: 'row', margin: 60 }}>
                <View style={{ flex: 1, margin: 10, padding: 10, backgroundColor: 'lightslategrey', }}>
                    <Button 
                        disabled={(!amountValid() || !event || creatingContract)}
                        title="Create" 
                        onPress={() => {
                            setCreatingContract(true);
                            dispatch(newContract(selectedPlayer.name, playerTwoName, parseInt(contractAmount), event, eventPayouts))
                                .then(() => navigation.reset({ index:0, routes: [{ name: 'Home' }] }))
                                .catch(error => console.error(error))
                                .finally(() => setCreatingContract(false));
                          } }
                    />
                </View>
            </View>
        </View>
    );
}

type EventInfoProps = {
    event: Event,
    payouts: string[];
    setEventPayouts: (payouts: string[]) => void,
}


const eventInfoStyles = StyleSheet.create({
    payout: {
        alignItems: "center",
        minWidth: 100,
    },
    outcomeText: {
        fontSize: 18 
    }
})

const EventInfo = (props: EventInfoProps) => {
    return (
        <View style={{ alignItems: "center"}}>
            <Text style={{fontSize: 18}}>Event</Text>
            <Text style={{fontSize: 20}}>{props.event.desc}</Text>
            <Text style={{fontSize: 18}}>Outcomes</Text>
            <View style={{ justifyContent: "space-between", flexDirection: "row" }}>
                <View style={eventInfoStyles.payout}>
                    <Text style={eventInfoStyles.outcomeText}>{props.event.outcomes[0].desc}</Text>
                    <Text>{props.payouts[0]}</Text>
                </View>
                <View style={{ margin: 5}}>
                    <Button 
                        title="<>" 
                        onPress={() => { 
                            let payouts = props.payouts.reverse();
                            props.setEventPayouts([...payouts]) 
                        }} 
                    />
                </View>
                <View style={eventInfoStyles.payout}>
                    <Text style={eventInfoStyles.outcomeText}>{props.event.outcomes[1].desc}</Text>
                    <Text>{props.payouts[1]}</Text>
                </View>
            </View>
        </View>
    )
}
